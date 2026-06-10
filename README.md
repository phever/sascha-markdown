# Sascha Flavored Markdown Editor (SFMDE)

Welcome to the **Sascha Flavored Markdown Editor**, a simple yet powerful tool designed for writing and organizing your thoughts. Whether you're taking notes, writing a blog post, or drafting a book, SFMDE provides a clean, modern interface that lets you focus on your words.

## What is "Sascha Flavored Markdown"?

You might be familiar with standard "Markdown"—a way to write text that automatically turns into beautiful formatting like **Bold**, *Italics*, or [Links](https://google.com).

**Sascha Flavored Markdown (SFM)** takes this a step further. It's a special version of Markdown that is **completely customizable**. Don't like using asterisks (`*`) for italics? You can change it to something else in your settings. Want a special symbol for spoilers or highlighted text? SFM lets you define exactly how your text should look.

## Key Features

*   **Dual-Pane Editing**: Write your text on the left and see a beautiful, live preview on the right instantly.
*   **Highly Customizable**: Redefine almost every formatting symbol to match your personal writing style.
*   **Modern Design**: Built with a sleek, "Adwaita" look that feels right at home on modern desktops.
*   **Dark Mode Support**: Automatically switches between light and dark themes to match your computer's settings—no more eye strain!
*   **Tabs**: Open several files at once, and the app stays open even with no file loaded. You'll be warned before closing anything with unsaved changes.
*   **Local-Only Mode**: By default the preview only loads images and links from your own computer—nothing is fetched from the internet unless you turn that on.
*   **Simple File Management**: Easily create, open, and save your `.smd` (Sascha Markdown) files. Use "Install Locally" from the welcome screen to make `.smd` files open with a double-click.

## How to Get Started

### 1. Installation
Since you are reading this, you likely found us on GitHub! 
1.  Go to the **Releases** section on the right side of the GitHub page.
2.  Download the latest version for your computer.
3.  Run the application! No complicated setup is required.

### 2. Your First Run
When you open SFMDE for the first time, you'll see a welcome message. The app will automatically create a settings folder on your computer where your personal preferences are stored.

### 3. Customizing Your Experience
If you want to change how the editor works (like changing your "Bold" symbol or how much history the app remembers):
1.  Navigate to your computer's "Config" folder (usually `.config/sascha-flavored-markdown/` on Linux).
2.  Open `sfmde.config` with any text editor.
3.  Change the symbols to your heart's content!

### 4. Writing Tips
-   **Only your symbols count**: formatting is driven entirely by the symbols in your settings. If you change Bold to something custom, a stray `**` from regular Markdown is shown as plain text instead of surprising you with bold.
-   **Lists inside lists**: indent the inner item by two spaces:
    ```
    - outer item
      - inner item
    ```
-   **Indentation and spacing are yours**: leading spaces and runs of multiple spaces are preserved in the preview instead of being collapsed (list indentation still nests as above).

## Need Help?
SFMDE is designed to be intuitive. If you get stuck, remember:
-   **Undo/Redo**: Use the arrows in the top left if you make a mistake.
-   **Settings**: The gear button opens Settings. "User" holds your personal defaults; "Local" saves overrides next to the current file (a `.smdconfig` file) that win over your defaults. Each formatter has a symbol, an *Enabled* switch (off = the symbol is treated as plain text), and a toolbar switch.
-   **Menu**: Click the menu button (three lines) for "About" and "Save As" options.

Happy Writing!
