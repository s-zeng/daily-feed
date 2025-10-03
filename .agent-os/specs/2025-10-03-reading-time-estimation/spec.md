# Spec Requirements Document

> Spec: Reading Time Estimation
> Created: 2025-10-03

## Overview

Implement reading time estimation that calculates and displays estimated reading time for articles, feeds, and the entire document. This feature will help readers plan their reading sessions and understand the time commitment before diving into content.

## User Stories

### Informed Reading Planning

As a daily-feed user, I want to see estimated reading times for articles, so that I can plan my reading sessions and prioritize content based on available time.

Users will see reading time estimates displayed in two locations: (1) in the article metadata at the top of each article, and (2) in the table of contents next to article titles. Additionally, users will see aggregated reading times for each feed and a total reading time for the entire document, helping them understand the full time commitment.

### Personalized Reading Speed

As a daily-feed user, I want to configure my reading speed, so that the time estimates match my actual reading pace.

Users can optionally configure their reading speed in words per minute in the config file. If not specified, the system will use a default of 200 words per minute. This accommodates different reading speeds and content complexity preferences.

## Spec Scope

1. **Word Count Calculation** - Calculate total words in article content blocks (paragraphs, headings, lists, quotes).
2. **Reading Time Calculation** - Convert word count to estimated reading time using configurable words-per-minute rate.
3. **Article-Level Display** - Show reading time in article metadata at the top of each article.
4. **Table of Contents Display** - Show reading time next to article titles in the table of contents.
5. **Feed-Level Aggregation** - Calculate and display total reading time per feed/source.
6. **Document-Level Aggregation** - Calculate and display total reading time for the entire EPUB/Markdown document.
7. **Configurable Reading Speed** - Support optional configuration of reading speed (default: 200 WPM).
8. **Time Formatting** - Display times as "X min" for under 60 minutes, "Xh Ymin" for longer content.

## Out of Scope

- Reading time tracking (actual time spent reading)
- Reading progress indicators
- Per-user reading speed calibration based on historical data
- Different reading speeds for different content types
- Reading time estimation for images, code blocks, or multimedia content

## Expected Deliverable

1. Articles, feeds, and the full document display accurate reading time estimates in both EPUB and Markdown outputs.
2. Configuration file supports optional `reading_speed_wpm` parameter that affects all time estimates.
3. Time formatting follows the pattern: "5 min", "45 min", "1h 15min", "2h 30min".
