# MALD Demo - Executable Notebook

This demonstrates MALD's Jupyter-style executable code cells in markdown.

## Python Example

```python
# Simple Python example
import datetime

current_time = datetime.datetime.now()
print(f"Current time: {current_time}")
print("MALD is running!")

# Let's do some basic calculations
result = 42 * 2
print(f"The answer to everything doubled: {result}")
```

<!-- MALD_OUTPUT_START -->
**Output:**
```
Current time: 2025-09-12 12:49:53.826373
MALD is running!
The answer to everything doubled: 84
```
<!-- MALD_OUTPUT_END -->

## Bash Commands

```bash
echo "Hello from MALD!"
echo "Current directory: $(pwd)"
echo "Files in current directory:"
ls -la
```

<!-- MALD_OUTPUT_START -->
**Output:**
```
Current directory: /home/runner/work/MALD/MALD
Files in current directory:
total 64
drwxr-xr-x 9 runner runner 4096 Sep 12 12:49 .
drwxr-xr-x 3 runner runner 4096 Sep 12 12:34 ..
drwxrwxr-x 7 runner runner 4096 Sep 12 12:49 .git
-rw-rw-r-- 1 runner runner  876 Sep 12 12:35 .gitignore
-rw-rw-r-- 1 runner runner 2569 Sep 12 12:35 CHANGELOG.md
-rw-rw-r-- 1 runner runner 1063 Sep 12 12:35 LICENSE
-rw-rw-r-- 1 runner runner 1681 Sep 12 12:35 Makefile
-rw-rw-r-- 1 runner runner 2666 Sep 12 12:35 README.md
drwxrwxr-x 2 runner runner 4096 Sep 12 12:35 bin
-rw-rw-r-- 1 runner runner 2033 Sep 12 12:49 demo.md
drwxrwxr-x 2 runner runner 4096 Sep 12 12:35 docs
drwxrwxr-x 2 runner runner 4096 Sep 12 12:35 iso
drwxrwxr-x 7 runner runner 4096 Sep 12 12:43 mald
-rw-rw-r-- 1 runner runner 2014 Sep 12 12:35 pyproject.toml
drwxrwxr-x 2 runner runner 4096 Sep 12 12:35 scripts
drwxrwxr-x 2 runner runner 4096 Sep 12 12:35 tests
```
<!-- MALD_OUTPUT_END -->

## JavaScript Example

```javascript
// JavaScript example
console.log("Hello from Node.js!");

const maldInfo = {
    name: "MALD",
    description: "Markdown Archive Linux Distribution",
    features: ["PKM", "RAG", "Executable Cells"]
};

console.log("MALD Info:", JSON.stringify(maldInfo, null, 2));
```

## SQL Example

```sql
-- Create a simple table
CREATE TABLE notes (
    id INTEGER PRIMARY KEY,
    title TEXT,
    content TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Insert some sample data
INSERT INTO notes (title, content) VALUES 
    ('Welcome to MALD', 'This is our first note'),
    ('RAG System', 'Retrieval Augmented Generation is cool'),
    ('Code Execution', 'Jupyter-style cells in markdown');

-- Query the data
SELECT * FROM notes ORDER BY created_at;
```

## MALD Features

This file demonstrates several key MALD features:

1. **Executable Code Cells**: Run code directly in markdown
2. **Multiple Languages**: Python, Bash, JavaScript, SQL
3. **Session Persistence**: Variables persist between cells (in Python)
4. **Output Capture**: See results inline

You can execute this file with:
```bash
mald exec file /path/to/this/file.md
```

Or execute individual cells:
```bash
mald exec cell /path/to/this/file.md python_123
```

Or list all cells:
```bash
mald exec list /path/to/this/file.md
```