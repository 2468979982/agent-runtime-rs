---
name: frontend-design
description: Generate frontend UI designs with HTML/CSS/JavaScript
version: 1.0.0
author: Agent Runtime RS
triggers:
  - design UI
  - create frontend
  - build interface
  - frontend design
tags:
  - frontend
  - design
  - UI
  - HTML
  - CSS
required_tools:
  - file_writer
  - file_reader
---

# Frontend Design Skill

This skill helps generate frontend UI designs using HTML, CSS, and JavaScript.

## Overview

Use this skill when you need to:
- Create a new UI design
- Generate HTML/CSS/JS code
- Build responsive web interfaces
- Prototype frontend components

## Usage

When the user requests a frontend design, follow these steps:

1. **Understand the requirements**: Ask about the purpose, target audience, and design preferences
2. **Plan the layout**: Sketch the basic layout (header, main content, footer, etc.)
3. **Generate the code**: Create HTML structure, CSS styles, and JavaScript functionality
4. **Save the files**: Use `file_writer` tool to save the generated code
5. **Preview**: Optionally create a simple preview server

## Example: Create a Login Page

### Step 1: Generate HTML

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Login - Agent Runtime RS</title>
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <div class="container">
        <div class="login-form">
            <h1>Login</h1>
            <form id="loginForm">
                <div class="form-group">
                    <label for="username">Username</label>
                    <input type="text" id="username" name="username" required>
                </div>
                <div class="form-group">
                    <label for="password">Password</label>
                    <input type="password" id="password" name="password" required>
                </div>
                <button type="submit" class="btn-primary">Login</button>
            </form>
        </div>
    </div>
    <script src="script.js"></script>
</body>
</html>
```

### Step 2: Generate CSS

```css
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
}

.container {
    width: 100%;
    max-width: 400px;
    padding: 20px;
}

.login-form {
    background: white;
    border-radius: 10px;
    padding: 40px;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.1);
}

.login-form h1 {
    text-align: center;
    margin-bottom: 30px;
    color: #333;
}

.form-group {
    margin-bottom: 20px;
}

.form-group label {
    display: block;
    margin-bottom: 5px;
    color: #555;
    font-weight: 500;
}

.form-group input {
    width: 100%;
    padding: 12px;
    border: 1px solid #ddd;
    border-radius: 5px;
    font-size: 14px;
    transition: border-color 0.3s;
}

.form-group input:focus {
    outline: none;
    border-color: #667eea;
}

.btn-primary {
    width: 100%;
    padding: 12px;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    border: none;
    border-radius: 5px;
    font-size: 16px;
    font-weight: 600;
    cursor: pointer;
    transition: transform 0.2s;
}

.btn-primary:hover {
    transform: translateY(-2px);
}

.btn-primary:active {
    transform: translateY(0);
}
```

### Step 3: Generate JavaScript

```javascript
document.getElementById('loginForm').addEventListener('submit', async (e) => {
    e.preventDefault();
    
    const username = document.getElementById('username').value;
    const password = document.getElementById('password').value;
    
    // Simulate API call
    console.log('Logging in...', { username });
    
    // Show loading state
    const btn = e.target.querySelector('.btn-primary');
    btn.textContent = 'Logging in...';
    btn.disabled = true;
    
    // Simulate network delay
    await new Promise(resolve => setTimeout(resolve, 1500));
    
    // Reset button
    btn.textContent = 'Login';
    btn.disabled = false;
    
    // Show success message
    alert('Login successful! (This is a demo)');
});
```

## Scripts

### Script 1: Create Login Page

Use this script to generate a complete login page with HTML, CSS, and JS.

**Language**: bash

```bash
#!/bin/bash
# This script creates a login page with HTML, CSS, and JS

OUTPUT_DIR="./output/frontend-design"
mkdir -p "$OUTPUT_DIR"

# Generate HTML
cat > "$OUTPUT_DIR/login.html" << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Login - Agent Runtime RS</title>
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <div class="container">
        <div class="login-form">
            <h1>Login</h1>
            <form id="loginForm">
                <div class="form-group">
                    <label for="username">Username</label>
                    <input type="text" id="username" name="username" required>
                </div>
                <div class="form-group">
                    <label for="password">Password</label>
                    <input type="password" id="password" name="password" required>
                </div>
                <button type="submit" class="btn-primary">Login</button>
            </form>
        </div>
    </div>
    <script src="script.js"></script>
</body>
</html>
EOF

# Generate CSS
cat > "$OUTPUT_DIR/style.css" << 'EOF'
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
}

/* ... (CSS code from above) ... */
EOF

# Generate JavaScript
cat > "$OUTPUT_DIR/script.js" << 'EOF'
document.getElementById('loginForm').addEventListener('submit', async (e) => {
    e.preventDefault();
    
    const username = document.getElementById('username').value;
    const password = document.getElementById('password').value;
    
    console.log('Logging in...', { username });
    
    const btn = e.target.querySelector('.btn-primary');
    btn.textContent = 'Logging in...';
    btn.disabled = true;
    
    await new Promise(resolve => setTimeout(resolve, 1500));
    
    btn.textContent = 'Login';
    btn.disabled = false;
    
    alert('Login successful! (This is a demo)');
});
EOF

echo "Login page created at: $OUTPUT_DIR"
echo "Files: login.html, style.css, script.js"
```

### Script 2: Create Dashboard

Use this script to generate a simple dashboard layout.

**Language**: python

```python
#!/usr/bin/env python3
# This script generates a dashboard HTML page

import os

output_dir = "./output/frontend-design"
os.makedirs(output_dir, exist_ok=True)

html_content = """
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Dashboard - Agent Runtime RS</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #f5f7fa;
        }
        
        .sidebar {
            width: 250px;
            height: 100vh;
            background: #2c3e50;
            color: white;
            position: fixed;
            left: 0;
            top: 0;
            padding: 20px;
        }
        
        .sidebar h2 {
            margin-bottom: 30px;
            font-size: 24px;
        }
        
        .sidebar ul {
            list-style: none;
        }
        
        .sidebar ul li {
            margin-bottom: 15px;
        }
        
        .sidebar ul li a {
            color: #ecf0f1;
            text-decoration: none;
            font-size: 16px;
            display: block;
            padding: 10px;
            border-radius: 5px;
            transition: background 0.3s;
        }
        
        .sidebar ul li a:hover {
            background: #34495e;
        }
        
        .main-content {
            margin-left: 250px;
            padding: 30px;
        }
        
        .header {
            margin-bottom: 30px;
        }
        
        .header h1 {
            color: #2c3e50;
            font-size: 32px;
        }
        
        .cards {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }
        
        .card {
            background: white;
            padding: 25px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
        }
        
        .card h3 {
            color: #7f8c8d;
            font-size: 14px;
            margin-bottom: 10px;
        }
        
        .card .value {
            color: #2c3e50;
            font-size: 32px;
            font-weight: bold;
        }
    </style>
</head>
<body>
    <div class="sidebar">
        <h2>Agent Runtime</h2>
        <ul>
            <li><a href="#">Dashboard</a></li>
            <li><a href="#">Projects</a></li>
            <li><a href="#">Tasks</a></li>
            <li><a href="#">Settings</a></li>
        </ul>
    </div>
    
    <div class="main-content">
        <div class="header">
            <h1>Dashboard</h1>
        </div>
        
        <div class="cards">
            <div class="card">
                <h3>Total Projects</h3>
                <div class="value">12</div>
            </div>
            <div class="card">
                <h3>Active Tasks</h3>
                <div class="value">47</div>
            </div>
            <div class="card">
                <h3>Completed</h3>
                <div class="value">128</div>
            </div>
        </div>
    </div>
</body>
</html>
"""

with open(os.path.join(output_dir, "dashboard.html"), "w") as f:
    f.write(html_content)

print(f"Dashboard created at: {output_dir}/dashboard.html")
```

## References

### Reference 1: CSS Grid Layout

Use CSS Grid for modern, responsive layouts:

```css
.container {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 20px;
}
```

### Reference 2: Flexbox Centering

Center elements horizontally and vertically:

```css
.container {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
}
```

### Reference 3: JavaScript Form Validation

Basic form validation example:

```javascript
function validateForm() {
    const email = document.getElementById('email').value;
    const password = document.getElementById('password').value;
    
    if (!email.includes('@')) {
        alert('Please enter a valid email');
        return false;
    }
    
    if (password.length < 8) {
        alert('Password must be at least 8 characters');
        return false;
    }
    
    return true;
}
```

## Integration with Agent Runtime

To use this skill in Agent Runtime:

1. **Load the skill**: The skill will be automatically loaded if `autoLoadSkills: true` in config
2. **Trigger the skill**: Use trigger phrases like "design UI" or "create frontend"
3. **Execute scripts**: Call the skill's scripts via the API:
   - `POST /api/skills/frontend-design/execute`
   - Body: `{ "script": "create-login-page", "arguments": {} }`

## Notes

- This skill requires `file_writer` and `file_reader` tools
- Generated files are saved to `./output/frontend-design/` by default
- Modify the scripts to change output location
