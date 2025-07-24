# Ars Technica Comment Loading Investigation

This doc written entire by Claude Code

## Overview
This document details how Ars Technica loads and displays comments on their articles, based on an investigation conducted on July 24, 2025.

## Key Findings

### Comment System Architecture
Ars Technica uses a **WordPress + XenForo hybrid system** for comments:
- Main articles are served from WordPress
- Comments are hosted on a separate XenForo forum (`/civis/` subdirectory)
- Comments are loaded via iframe embedding from the forum into the article page

### Comment Loading Mechanism

#### 1. Initial Page Load
- Article pages contain a placeholder div: `<div class="wp-forum-connect-comments relative">`
- Comments section is initially hidden with CSS class `hidden`
- A "Comments" link points to `#comments` anchor with comment count (e.g., "28 Comments")

#### 2. Comment Loading Trigger
Comments are loaded when:
- User clicks a "View Comments" link
- URL contains `#comments` hash
- URL contains `#respond` hash
- URL has query parameter containing "comments"

#### 3. Iframe Loading Process
When triggered, JavaScript (`iframe.js`) performs these steps:

1. **Show Loading State**: Displays loading indicator and unhides comments wrapper
2. **Create Iframe**: Dynamically creates an iframe element with:
   - ID: `xf_thread_iframe`
   - Source URL from `data-url` attribute
   - Frameborder: 0, Width: 100%

3. **Iframe Source URL Structure**:
   ```
   https://arstechnica.com/civis/threads/[article-slug].[thread-id]/unread?in_iframe=1&theme=auto&wp_data=[base64_encoded_data]
   ```

#### 4. Data Exchange
- **wp_data parameter**: Base64-encoded JSON containing:
  ```json
  {
    "url": "https://arstechnica.com/science/2025/07/marine-biologist-for-a-day-ars-goes-shark-tagging/",
    "open_comments": "comments=1"
  }
  ```
- **Communication**: Uses `iframe-resizer` library for bidirectional communication between parent and iframe
- **Scroll Management**: Tracks user scroll position and syncs with iframe content

### Technical Implementation Details

#### JavaScript Files Involved
1. **`iframe.js`** (v1.2.4): Main comment loading logic
2. **`iframe-resizer.parent.js`** (v5.3.1): Handles iframe resizing and communication

#### WordPress Plugin
- Uses `article-forum-connect` plugin to bridge WordPress articles with XenForo forum threads
- Plugin handles iframe creation, URL generation, and user authentication sync

#### URL Patterns
- **Article URL**: `https://arstechnica.com/[category]/[year]/[month]/[article-slug]/`
- **Forum Thread**: `https://arstechnica.com/civis/threads/[article-slug].[thread-id]/`
- **Iframe URL**: Forum thread URL + `unread?in_iframe=1&theme=auto&wp_data=[encoded_data]`

### Comment Threading System
- Comments use XenForo's native threading and pagination
- Support for user authentication, voting, and moderation
- Real-time updates through iframe communication
- Maintains scroll position and handles navigation within comments

### Accessibility & UX Features
- **Responsive Design**: Comments adapt to different screen sizes
- **Theme Support**: Supports light/dark themes via `theme=auto` parameter  
- **URL State**: Comment page navigation updates browser URL without page reload
- **Deep Linking**: Direct links to specific comments supported via URL parameters

## Testing with curl

### Basic Article Fetch
```bash
curl -s "https://arstechnica.com/science/2025/07/marine-biologist-for-a-day-ars-goes-shark-tagging/"
```

### Comment Data Extraction
```bash
# Find comment count and links
curl -s "[article_url]" | grep -i comment

# Extract iframe URL
curl -s "[article_url]" | grep -oE 'data-url="[^"]*"'

# Decode wp_data parameter
echo "[base64_string]" | base64 -d
```

### Forum Thread Access
```bash
# Access forum thread directly
curl -s "https://arstechnica.com/civis/threads/[article-slug].[thread-id]/"
```

## Conclusion

Ars Technica uses a sophisticated commenting system that separates content (WordPress) from community features (XenForo). The iframe-based approach allows them to maintain separate systems while providing a seamless user experience. The system handles authentication, theming, and navigation state management through careful JavaScript coordination between the parent page and embedded forum content.

This hybrid approach allows Ars Technica to leverage XenForo's robust forum features while maintaining their custom article design and CMS workflow.
