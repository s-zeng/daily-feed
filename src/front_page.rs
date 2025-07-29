use crate::ai_client::{AiClient, AiProvider, AiClientError};
use crate::ast::{Document, Headline, ContentBlock, TextContent, TextSpan, TextFormatting};
use std::error::Error;
use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum FrontPageError {
    AiError(AiClientError),
    GenerationError(String),
    ParseError(String),
}

impl fmt::Display for FrontPageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FrontPageError::AiError(e) => write!(f, "AI error: {}", e),
            FrontPageError::GenerationError(msg) => write!(f, "Generation error: {}", msg),
            FrontPageError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl Error for FrontPageError {}

impl From<AiClientError> for FrontPageError {
    fn from(error: AiClientError) -> Self {
        FrontPageError::AiError(error)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredFrontPage {
    pub theme: String,
    pub stories: Vec<TopStory>,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopStory {
    pub title: String,
    pub summary: String,
    pub impact: String,
}

pub struct FrontPageGenerator {
    ai_client: AiClient,
}

impl FrontPageGenerator {
    pub fn new(provider: AiProvider) -> Result<Self, FrontPageError> {
        let ai_client = AiClient::new(provider)?;
        Ok(FrontPageGenerator { ai_client })
    }

    pub async fn generate_structured_front_page(&self, headlines: &[Headline]) -> Result<Vec<ContentBlock>, FrontPageError> {
        let structured_data = self.generate_structured_data(headlines).await?;
        Ok(self.convert_to_ast(&structured_data))
    }

    pub async fn generate_structured_front_page_from_document(&self, document: &Document) -> Result<Vec<ContentBlock>, FrontPageError> {
        let headlines = document.extract_headlines();
        self.generate_structured_front_page(&headlines).await
    }

    async fn generate_structured_data(&self, headlines: &[Headline]) -> Result<StructuredFrontPage, FrontPageError> {
        let content = self.prepare_content(headlines)?;
        let prompt = self.build_structured_prompt(&content);
        
        let response = self.ai_client.generate_text(&prompt).await?;
        self.parse_structured_response(&response)
    }

    pub fn convert_to_ast(&self, front_page: &StructuredFrontPage) -> Vec<ContentBlock> {
        let mut blocks = Vec::new();
        
        // Add theme as opening paragraph
        blocks.push(ContentBlock::Paragraph(TextContent::from_spans(vec![
            TextSpan {
                text: "Today's World: ".to_string(),
                formatting: TextFormatting { bold: true, ..Default::default() },
            },
            TextSpan::plain(front_page.theme.clone()),
        ])));

        // Add heading for top stories
        blocks.push(ContentBlock::Heading {
            level: 2,
            content: TextContent::plain("Top Stories".to_string()),
        });

        // Add each story as a list item with structured content
        let story_items: Vec<TextContent> = front_page.stories.iter().map(|story| {
            TextContent::from_spans(vec![
                TextSpan {
                    text: format!("{}: ", story.title),
                    formatting: TextFormatting { bold: true, ..Default::default() },
                },
                TextSpan::plain(format!("{} ", story.summary)),
                TextSpan {
                    text: story.impact.clone(),
                    formatting: TextFormatting { italic: true, ..Default::default() },
                },
            ])
        }).collect();

        blocks.push(ContentBlock::List {
            ordered: false,
            items: story_items,
        });

        // Add context if present
        if let Some(context) = &front_page.context {
            blocks.push(ContentBlock::Heading {
                level: 3,
                content: TextContent::plain("Looking Ahead".to_string()),
            });
            blocks.push(ContentBlock::Paragraph(TextContent::plain(context.clone())));
        }

        blocks
    }

    pub fn parse_structured_response(&self, response: &str) -> Result<StructuredFrontPage, FrontPageError> {
        // First, try to extract JSON from markdown code blocks
        let json_content = self.extract_json_from_response(response);
        
        // Try to parse as JSON first
        if let Ok(structured) = serde_json::from_str::<StructuredFrontPage>(&json_content) {
            return Ok(structured);
        }

        // If JSON parsing fails, try to extract structured data from markdown-like format
        self.parse_markdown_response(response)
    }

    pub fn extract_json_from_response(&self, response: &str) -> String {
        // Look for JSON in markdown code blocks first
        if let Some(start) = response.find("```json") {
            let after_start = &response[start + 7..]; // Skip "```json"
            if let Some(end) = after_start.find("```") {
                return after_start[..end].trim().to_string();
            }
        }
        
        // Look for JSON in generic code blocks
        if let Some(start) = response.find("```") {
            let after_start = &response[start + 3..];
            if let Some(end) = after_start.find("```") {
                let content = after_start[..end].trim();
                // Check if this looks like JSON (starts with { and ends with })
                if content.trim_start().starts_with('{') && content.trim_end().ends_with('}') {
                    return content.to_string();
                }
            }
        }
        
        // Look for standalone JSON objects (lines starting with { and ending with })
        let lines: Vec<&str> = response.lines().collect();
        let mut json_start = None;
        let mut json_end = None;
        let mut brace_count = 0;
        
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if json_start.is_none() && trimmed.starts_with('{') {
                json_start = Some(i);
                brace_count = 1;
                // Count braces in the first line
                for ch in trimmed.chars().skip(1) {
                    match ch {
                        '{' => brace_count += 1,
                        '}' => brace_count -= 1,
                        _ => {}
                    }
                }
                if brace_count == 0 {
                    json_end = Some(i);
                    break;
                }
            } else if json_start.is_some() && brace_count > 0 {
                // Count braces in subsequent lines
                for ch in trimmed.chars() {
                    match ch {
                        '{' => brace_count += 1,
                        '}' => brace_count -= 1,
                        _ => {}
                    }
                }
                if brace_count == 0 {
                    json_end = Some(i);
                    break;
                }
            }
        }
        
        if let (Some(start), Some(end)) = (json_start, json_end) {
            return lines[start..=end].join("\n");
        }
        
        // If no JSON found, return original response
        response.to_string()
    }

    fn parse_markdown_response(&self, response: &str) -> Result<StructuredFrontPage, FrontPageError> {
        let lines: Vec<&str> = response.lines().collect();
        let mut theme = String::new();
        let mut stories = Vec::new();
        let mut context = None;
        
        let mut current_section = "theme";
        let mut current_story: Option<TopStory> = None;
        
        for line in lines {
            let line = line.trim();
            
            if line.is_empty() {
                continue;
            }
            
            // Detect section headers
            if line.contains("Today's World") || line.contains("**Today's World**") {
                current_section = "theme";
                // Extract theme from same line if present
                let clean_line = line
                    .replace("**Today's World**:", "")
                    .replace("**Today's World**: ", "")
                    .replace("Today's World:", "")
                    .replace("Today's World: ", "")
                    .trim()
                    .to_string();
                if !clean_line.is_empty() {
                    theme = clean_line;
                }
                continue;
            } else if line.contains("Top Stories") || line.contains("**Top Stories**") {
                current_section = "stories";
                continue;
            } else if line.contains("Looking Ahead") || line.contains("**Looking Ahead**") {
                current_section = "context";
                continue;
            }
            
            match current_section {
                "theme" => {
                    if !theme.is_empty() {
                        theme.push(' ');
                    }
                    let clean_line = line
                        .replace("**Today's World**:", "")
                        .replace("**Today's World**: ", "")
                        .replace("Today's World:", "")
                        .replace("Today's World: ", "")
                        .trim()
                        .to_string();
                    if !clean_line.is_empty() {
                        theme.push_str(&clean_line);
                    }
                }
                "stories" => {
                    if line.starts_with("• **") || line.starts_with("- **") || line.starts_with("* **") {
                        // Save previous story if exists
                        if let Some(story) = current_story.take() {
                            stories.push(story);
                        }
                        
                        // Extract title from bullet point
                        let without_bullet = line.trim_start_matches(&['•', '-', '*', ' '][..]).trim();
                        // Look for pattern: **Title**: content
                        if let Some(first_star) = without_bullet.find("**") {
                            let after_first_star = &without_bullet[first_star + 2..];
                            if let Some(title_end) = after_first_star.find("**:") {
                                let title = after_first_star[..title_end].to_string();
                                let rest = after_first_star[title_end + 3..].trim().to_string();
                            
                                current_story = Some(TopStory {
                                    title,
                                    summary: rest,
                                    impact: String::new(),
                                });
                            }
                        }
                    } else if let Some(ref mut story) = current_story {
                        // Continue building the story content
                        if !story.summary.is_empty() {
                            story.summary.push(' ');
                        }
                        story.summary.push_str(line);
                    }
                }
                "context" => {
                    if context.is_none() {
                        context = Some(String::new());
                    }
                    if let Some(ref mut ctx) = context {
                        if !ctx.is_empty() {
                            ctx.push(' ');
                        }
                        ctx.push_str(line);
                    }
                }
                _ => {}
            }
        }
        
        // Save the last story
        if let Some(story) = current_story {
            stories.push(story);
        }
        
        if theme.is_empty() && stories.is_empty() {
            return Err(FrontPageError::ParseError(
                "Could not parse structured front page from AI response".to_string()
            ));
        }
        
        Ok(StructuredFrontPage {
            theme: if theme.is_empty() { "Multiple developing stories shape today's landscape".to_string() } else { theme },
            stories,
            context,
        })
    }

    pub fn build_structured_prompt(&self, content: &str) -> String {
        format!(
            r#"You are a senior news editor creating a structured "Front Page" summary from daily news feeds. 

Analyze the provided content and return a JSON response with this exact structure:

{{
  "theme": "One sentence capturing the day's most significant theme or development",
  "stories": [
    {{
      "title": "Clear, descriptive headline",
      "summary": "1-2 sentences explaining what happened and why it matters",
      "impact": "Brief note on who is affected or what comes next"
    }}
  ],
  "context": "Optional sentence connecting stories to broader trends"
}}

Guidelines:
- Identify 3-5 most important stories
- Prioritize breaking news, high impact, and controversial stories
- Keep summaries concise but informative
- Maintain neutral tone
- Focus on stories affecting large populations or having long-term implications

Daily feed content:
{}

Return only valid JSON with the structure above."#,
            content
        )
    }

    pub async fn generate_front_page(&self, headlines: &[Headline]) -> Result<String, FrontPageError> {
        let content = self.prepare_content(headlines)?;
        let prompt = self.build_prompt(&content);
        
        let front_page = self.ai_client.generate_text(&prompt).await?;
        Ok(front_page)
    }

    pub async fn generate_front_page_from_document(&self, document: &Document) -> Result<String, FrontPageError> {
        let headlines = document.extract_headlines();
        self.generate_front_page(&headlines).await
    }

    pub fn prepare_content(&self, headlines: &[Headline]) -> Result<String, FrontPageError> {
        let mut content = String::new();
        
        for headline in headlines {
            content.push_str(&format!("## {}\n", headline.title));
            content.push_str(&format!("**Source:** {}\n", headline.source_name));
            
            if let Some(date) = &headline.published_date {
                content.push_str(&format!("**Published:** {}\n", date));
            }
            
            if let Some(url) = &headline.url {
                content.push_str(&format!("**URL:** {}\n", url));
            }
            
            content.push_str("\n");
        }
        
        Ok(content)
    }

    pub fn build_prompt(&self, content: &str) -> String {
        format!(
            r#"You are a senior news editor tasked with creating a concise "Front Page" summary from a daily news feed. Your goal is to identify the 3-5 most important stories and present them in a way that gives readers a quick understanding of the current state of the world.

Analyze the provided daily feed content and create a front page summary following these guidelines:

### Content Analysis
- **Identify Breaking News**: Look for stories marked as recent or urgent
- **Assess Impact**: Prioritize stories affecting large populations or having long-term implications
- **Spot Controversies**: Highlight stories generating significant debate or multiple perspectives
- **Find Connections**: Group related stories and identify overarching themes
- **Filter Noise**: Exclude minor local incidents, celebrity gossip, and repetitive updates

### Output Structure
1. **Opening Line**: One sentence capturing the day's most significant theme or development
2. **Top Stories** (3-5 items):
   - **Story Title**: Clear, descriptive headline
   - **Summary**: 1-2 sentences explaining what happened and why it matters
   - **Impact**: Brief note on who is affected or what comes next
3. **World Context**: One sentence connecting these stories to broader trends if applicable

### Writing Style
- **Neutral Tone**: Present facts without editorial commentary
- **Concise Language**: Maximum 350 words total
- **Clear Structure**: Use bullet points or numbered lists for readability
- **Accessible**: Avoid jargon; explain technical terms when necessary
- **Urgent**: Convey importance and timeliness

### Example Format

**Today's World**: [Single sentence theme]

**Top Stories:**
• **[Story 1 Title]**: [What happened]. [Why it matters/who's affected].
• **[Story 2 Title]**: [What happened]. [Why it matters/who's affected].
• **[Story 3 Title]**: [What happened]. [Why it matters/who's affected].

**Looking Ahead**: [Optional: upcoming developments or implications]

Please create a Front Page summary from the following daily feed content. Focus on the most important and controversial stories that give the best overview of today's world:

{}

Remember to:
- Lead with the most impactful story
- Keep each summary to 1-2 sentences
- Explain why stories matter, not just what happened
- Maintain neutral tone while acknowledging controversy
- Stay under 350 words total"#,
            content
        )
    }
}
