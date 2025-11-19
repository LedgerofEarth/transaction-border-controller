
# TBC / CoreProver Development Cheat Sheet

This cheat sheet provides commonly used development commands for working with the `transaction-border-controller` repository.  
It is designed as a reference for developers and contributors (including Shannon) and uses **generic file names and test names**.

—

## 🚀 Basic Cargo Commands

### **Build the entire workspace**
```bash
cargo build —workspace
```

### **Clean compiled artifacts**
```bash
cargo clean
```

—

## 🧪 Running Tests

### **Run all tests in the workspace**
```bash
cargo test —workspace
```

### **Run tests for a specific crate**
```bash
cargo test -p <crate-name>
```

### **Run a specific integration test file**
```bash
cargo test -p <crate-name> —test <file_name>
```

### **Run a single test function**
```bash
cargo test -p <crate-name> <test_name> — —exact
```

### **List all available tests in a crate**
```bash
cargo test -p <crate-name> — —list
```

### **Show test output (println!)**
```bash
cargo test -p <crate-name> — —nocapture
```

—

## 📄 Capturing Test Output to a File

### **Save stdout + stderr from a specific test**
```bash
cargo test -p <crate-name> —test <file_name> — —nocapture > output.txt 2>&1
cargo test --workspace --all-features -- --nocapture 2>&1 | tee test-output.log
```

### **View the saved output**
```bash
cat output.txt
```
cargo test --workspace --all-targets --all-features > test_output.txt 2>&1

—

## 🔍 Git & Repo Management

### **Pull latest changes from main**
```bash
git checkout main
git pull origin main
```

### **Reset local to match remote (⚠ destructive)**
```bash
git fetch origin
git reset —hard origin/main
```

### **Update branch with rebase**
```bash
git pull origin main —rebase
```
tree -L 4 crates > tree_creates.txt
—

## 🧭 Simulation Test Shortcuts

### **Pizza Simulation**
```bash
cargo test -p coreprover-service —test sim_pizza — —nocapture
```

### **Swap Simulation**
```bash
cargo test -p coreprover-service —test sim_swap — —nocapture
```

### **Purchase Simulation**
```bash
cargo test -p coreprover-service —test sim_purchase — —nocapture
```

—

## 🗂 File & Directory Commands

### **List directory contents**
```bash
ls -l
```

### **Open file in Codespace editor**
```bash
code <filename>
```

### **Delete file**
```bash
rm <filename>
```

### **Delete folder**
```bash
rm -rf <foldername>
```

—

## 🌐 Useful Development Patterns

### **Clean + rebuild**
```bash
cargo clean && cargo build —workspace
```

### **Rerun only failed tests**
```bash
cargo test — —failed
cargo test -p coreprover-service --test pizza_sim -- --nocapture 2>&1 | tee test_output.txt

## 📦 Replaceable Placeholders

- `<crate-name>` → e.g., `coreprover-service`, `coreprover-bridge`, `coreprover-sdk`
- `<file_name>` → e.g., `sim_swap`, `sim_pizza`
- `<test_name>` → single test function name

—

## 📘 Notes for Contributors

- Use `—nocapture` for debugging simulation output.
- Simulation tests reside in:  
  `crates/<crate-name>/tests/`
- Safe to add this file to repo as: **CHEATSHEET.md**

