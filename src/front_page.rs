use crate::ai_client::{AiClient, AiClientError, AiProvider};
use crate::ast::{ContentBlock, Document, TextContent, TextFormatting, TextSpan};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

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
    pub sources: Vec<SourceSummary>,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSummary {
    pub name: String,
    pub summary: String,
    pub key_stories: Vec<String>,
}

pub struct FrontPageGenerator {
    ai_client: AiClient,
}

impl FrontPageGenerator {
    pub fn new(provider: AiProvider) -> Result<Self, FrontPageError> {
        let ai_client = AiClient::new(provider)?;
        Ok(FrontPageGenerator { ai_client })
    }

    pub async fn generate_structured_front_page_from_document(
        &self,
        document: &Document,
    ) -> Result<Vec<ContentBlock>, FrontPageError> {
        let structured_data = self.generate_structured_data_by_source(document).await?;
        Ok(self.convert_to_ast(&structured_data))
    }

    async fn generate_structured_data_by_source(
        &self,
        document: &Document,
    ) -> Result<StructuredFrontPage, FrontPageError> {
        let content = self.prepare_content_by_source(document)?;
        let prompt = self.build_structured_prompt_by_source(&content);

        let response = self.ai_client.generate_text(&prompt).await?;
        self.parse_structured_response_by_source(&response)
    }

    pub fn convert_to_ast(&self, front_page: &StructuredFrontPage) -> Vec<ContentBlock> {
        let mut blocks = Vec::new();

        // Add theme as opening paragraph
        blocks.push(ContentBlock::Paragraph(TextContent::from_spans(vec![
            TextSpan {
                text: "Today's World: ".to_string(),
                formatting: TextFormatting {
                    bold: true,
                    ..Default::default()
                },
            },
            TextSpan::plain(front_page.theme.clone()),
        ])));

        // Add each source summary
        for source in &front_page.sources {
            // Add source heading
            blocks.push(ContentBlock::Heading {
                level: 2,
                content: TextContent::plain(source.name.clone()),
            });

            // Add source summary
            blocks.push(ContentBlock::Paragraph(TextContent::plain(
                source.summary.clone(),
            )));

            // Add key stories if present
            if !source.key_stories.is_empty() {
                blocks.push(ContentBlock::Heading {
                    level: 3,
                    content: TextContent::plain("Key Stories".to_string()),
                });

                let story_items: Vec<TextContent> = source
                    .key_stories
                    .iter()
                    .map(|story| TextContent::plain(story.clone()))
                    .collect();

                blocks.push(ContentBlock::List {
                    ordered: false,
                    items: story_items,
                });
            }
        }

        // Add context if present
        if let Some(context) = &front_page.context {
            blocks.push(ContentBlock::Heading {
                level: 2,
                content: TextContent::plain("Looking Ahead".to_string()),
            });
            blocks.push(ContentBlock::Paragraph(TextContent::plain(context.clone())));
        }

        blocks
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

    pub fn parse_structured_response_by_source(
        &self,
        response: &str,
    ) -> Result<StructuredFrontPage, FrontPageError> {
        // First, try to extract JSON from markdown code blocks
        let json_content = self.extract_json_from_response(response);

        // Try to parse as JSON first
        if let Ok(structured) = serde_json::from_str::<StructuredFrontPage>(&json_content) {
            return Ok(structured);
        }

        // If JSON parsing fails, try to extract structured data from markdown-like format
        self.parse_markdown_response_by_source(response)
    }

    fn parse_markdown_response_by_source(
        &self,
        response: &str,
    ) -> Result<StructuredFrontPage, FrontPageError> {
        let lines: Vec<&str> = response.lines().collect();
        let mut theme = String::new();
        let mut sources = Vec::new();
        let mut context = None;

        let mut current_section = "theme";
        let mut current_source: Option<SourceSummary> = None;

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
            } else if line.starts_with("##")
                || line.contains("**") && !line.contains("Looking Ahead")
            {
                // This might be a source name
                if let Some(source) = current_source.take() {
                    sources.push(source);
                }

                let source_name = line
                    .replace("##", "")
                    .replace("**", "")
                    .replace(":", "")
                    .trim()
                    .to_string();

                current_source = Some(SourceSummary {
                    name: source_name,
                    summary: String::new(),
                    key_stories: Vec::new(),
                });
                current_section = "source";
                continue;
            } else if line.contains("Looking Ahead") || line.contains("**Looking Ahead**") {
                if let Some(source) = current_source.take() {
                    sources.push(source);
                }
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
                "source" => {
                    if let Some(ref mut source) = current_source {
                        if line.starts_with("• ")
                            || line.starts_with("- ")
                            || line.starts_with("* ")
                        {
                            // This is a key story
                            let story = line
                                .trim_start_matches(&['•', '-', '*', ' '][..])
                                .trim()
                                .to_string();
                            source.key_stories.push(story);
                        } else {
                            // This is part of the summary
                            if !source.summary.is_empty() {
                                source.summary.push(' ');
                            }
                            source.summary.push_str(line);
                        }
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

        // Save the last source
        if let Some(source) = current_source {
            sources.push(source);
        }

        if theme.is_empty() && sources.is_empty() {
            return Err(FrontPageError::ParseError(
                "Could not parse structured front page from AI response".to_string(),
            ));
        }

        Ok(StructuredFrontPage {
            theme: if theme.is_empty() {
                "Multiple developing stories shape today's landscape".to_string()
            } else {
                theme
            },
            sources,
            context,
        })
    }

    pub fn build_structured_prompt_by_source(&self, content: &str) -> String {
        format!(
            r#"You are a senior news editor creating a structured "Front Page" summary organized by news sources. 

Analyze the provided content and return a JSON response with this exact structure:

{{
  "theme": "One sentence capturing the day's most significant theme or development across all sources",
  "sources": [
    {{
      "name": "Source name",
      "summary": "2-3 sentences summarizing the main themes and developments from this source",
      "key_stories": ["Key story title 1", "Key story title 2", "Key story title 3"]
    }}
  ],
  "context": "Optional sentence connecting stories across sources to broader trends"
}}

Guidelines:
- For each source, provide a thematic summary of their coverage
- Include 2-4 most important story titles from each source
- Maintain neutral tone
- Focus on what each source is emphasizing or covering uniquely
- Keep source summaries concise but informative
- The overall theme should reflect patterns across all sources

Daily feed content organized by source:
{}

Return only valid JSON with the structure above."#,
            content
        )
    }

    pub fn prepare_content_by_source(&self, document: &Document) -> Result<String, FrontPageError> {
        let mut content = String::new();

        for feed in &document.feeds {
            content.push_str(&format!("# Source: {}\n", feed.name));

            if let Some(description) = &feed.description {
                content.push_str(&format!("**Description:** {}\n", description));
            }

            if let Some(url) = &feed.url {
                content.push_str(&format!("**URL:** {}\n", url));
            }

            content.push_str("\n**Articles:**\n");

            for article in &feed.articles {
                content.push_str(&format!("- {}", article.title));

                if let Some(date) = &article.metadata.published_date {
                    content.push_str(&format!(" ({})", date));
                }

                content.push_str("\n");
            }

            content.push_str("\n");
        }

        Ok(content)
    }
}
