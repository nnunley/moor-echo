#!/usr/bin/env python3
"""
Echo Language Test Runner
Run Echo test files from the command line
"""

import subprocess
import sys
import os
import re
import tempfile
from pathlib import Path

# ANSI color codes
GREEN = '\033[0;32m'
RED = '\033[0;31m'
YELLOW = '\033[0;33m'
BLUE = '\033[0;34m'
NC = '\033[0m'  # No Color

class EchoTestRunner:
    def __init__(self, db_dir):
        self.echo_repl = ["cargo", "run", "--quiet", "--bin", "echo-repl", "--"]
        self.db_dir = db_dir
        self.echo_repl.extend(["--db", self.db_dir])
        self.total_tests = 0
        self.passed_tests = 0
        self.failed_tests = 0
        
    def strip_comments(self, content):
        """Remove comment lines from Echo code"""
        lines = content.split('\n')
        cleaned_lines = []
        for line in lines:
            # Remove lines that start with // (after optional whitespace)
            if not re.match(r'^\s*//', line):
                cleaned_lines.append(line)
        return '\n'.join(cleaned_lines)
    
    def prepare_test_content(self, test_file):
        """Prepare test file content by removing comments and handling .load commands"""
        with open(test_file, 'r') as f:
            content = f.read()
        
        # Remove comments
        content = self.strip_comments(content)
        
        # Handle .load commands by inlining the content
        # This is a simple implementation that doesn't handle nested .load
        lines = content.split('\n')
        expanded_lines = []
        
        for line in lines:
            if line.strip().startswith('.load '):
                # Extract the file path
                load_match = re.match(r'\.load\s+(.+)', line.strip())
                if load_match:
                    load_path = load_match.group(1).strip()
                    # Resolve relative to test file directory
                    test_dir = os.path.dirname(test_file)
                    full_load_path = os.path.join(test_dir, load_path)
                    
                    if os.path.exists(full_load_path):
                        print(f"  Inlining: {load_path}")
                        with open(full_load_path, 'r') as f:
                            loaded_content = self.strip_comments(f.read())
                            expanded_lines.append(f"// Inlined from {load_path}")
                            expanded_lines.append(loaded_content)
                    else:
                        print(f"  {RED}Warning: Could not load {load_path}{NC}")
                        expanded_lines.append(line)
                else:
                    expanded_lines.append(line)
            else:
                expanded_lines.append(line)
        
        return '\n'.join(expanded_lines)
    
    def run_echo_code(self, code):
        """Run Echo code through the REPL and return output"""
        try:
            # If code doesn't start with .eval, wrap it
            if not code.strip().startswith('.eval'):
                code = '.eval\n' + code + '\n.'
            
            # Create a temporary file for the code
            with tempfile.NamedTemporaryFile(mode='w', suffix='.echo', delete=False) as tmp:
                tmp.write(code)
                tmp.write('\n.quit\n')  # Ensure REPL exits
                tmp_path = tmp.name
            
            # Run the Echo REPL with the code
            result = subprocess.run(
                self.echo_repl,
                stdin=open(tmp_path, 'r'),
                capture_output=True,
                text=True,
                timeout=30
            )
            
            # Clean up temp file
            os.unlink(tmp_path)
            
            return result.stdout, result.stderr, result.returncode
            
        except subprocess.TimeoutExpired:
            os.unlink(tmp_path)
            return "", "Test timed out after 30 seconds", 1
        except Exception as e:
            if 'tmp_path' in locals():
                os.unlink(tmp_path)
            return "", str(e), 1
    
    def extract_test_results(self, output):
        """Extract test results from Echo output"""
        # Look for test summary patterns
        passed_match = re.search(r'Passed:\s*(\d+)', output)
        failed_match = re.search(r'Failed:\s*(\d+)', output)
        
        passed = int(passed_match.group(1)) if passed_match else 0
        failed = int(failed_match.group(1)) if failed_match else 0
        
        # Also check for the simple test format
        if not passed_match and not failed_match:
            # Count ✓ and ✗ symbols
            passed = output.count('✓')
            failed = output.count('✗')
        
        return passed, failed
    
    def run_test_file(self, test_file):
        """Run a single test file"""
        test_name = os.path.basename(test_file)
        print(f"{YELLOW}Running test: {test_name}{NC}")
        
        # Check if this is a simple inline test or needs preparation
        with open(test_file, 'r') as f:
            content = f.read()
        
        # Check if file has .load commands or comments
        if '.load' in content or '//' in content:
            print(f"  Preparing test file...")
            code = self.prepare_test_content(test_file)
        else:
            code = content
        
        # Run the test
        stdout, stderr, returncode = self.run_echo_code(code)
        
        # Analyze results
        if stderr:
            print(f"{RED}  Error output:{NC}")
            print(f"  {stderr}")
        
        passed, failed = self.extract_test_results(stdout)
        
        # Check for success
        success = (returncode == 0 and 
                  (failed == 0 or "✅ All tests passed!" in stdout) and
                  passed > 0)
        
        if success:
            print(f"{GREEN}✓ {test_name}: PASSED ({passed} tests){NC}")
            self.passed_tests += 1
        else:
            print(f"{RED}✗ {test_name}: FAILED{NC}")
            if passed > 0 or failed > 0:
                print(f"  Tests: {passed} passed, {failed} failed")
            print(f"{BLUE}Output:{NC}")
            print(stdout)
            self.failed_tests += 1
        
        self.total_tests += 1
        print()
        
        return success
    
    def run_rust_tests(self):
        """Run Rust unit tests"""
        print(f"{YELLOW}Running Rust unit tests...{NC}")
        
        result = subprocess.run(
            ["cargo", "test", "--", "--nocapture"],
            capture_output=True,
            text=True
        )
        
        if result.returncode == 0:
            print(f"{GREEN}✓ Rust tests: PASSED{NC}")
            # Extract test count from cargo test output
            test_match = re.search(r'test result: ok\. (\d+) passed', result.stdout)
            if test_match:
                rust_passed = int(test_match.group(1))
                print(f"  {rust_passed} tests passed")
        else:
            print(f"{RED}✗ Rust tests: FAILED{NC}")
            print(result.stdout)
            print(result.stderr)
        
        print()
        return result.returncode == 0
    
    def run_all_tests(self, test_files=None, run_rust_tests=True):
        """Run all tests or specific test files"""
        print("Echo Language Test Runner")
        print("=" * 25)
        print(f"Using database: {self.db_dir}")
        print()
        
        # Run Rust tests if requested
        rust_success = True
        if run_rust_tests:
            rust_success = self.run_rust_tests()
        
        # Determine which Echo test files to run
        if test_files:
            # Run specific test files
            for test_file in test_files:
                if os.path.exists(test_file):
                    self.run_test_file(test_file)
                else:
                    print(f"{RED}Error: Test file not found: {test_file}{NC}")
        else:
            # Run all test files in order
            test_locations = [
                "mini_test.echo",
                "echo_test.echo",
                "simple_test.echo",
                "tests/*.echo"
            ]
            
            for pattern in test_locations:
                if '*' in pattern:
                    # Handle glob pattern
                    from glob import glob
                    for test_file in sorted(glob(pattern)):
                        if os.path.exists(test_file):
                            self.run_test_file(test_file)
                else:
                    # Single file
                    if os.path.exists(pattern):
                        self.run_test_file(pattern)
        
        # Summary
        print("=" * 25)
        print("Test Summary:")
        print(f"  Echo Tests: {self.total_tests}")
        print(f"  {GREEN}Passed: {self.passed_tests}{NC}")
        print(f"  {RED}Failed: {self.failed_tests}{NC}")
        if run_rust_tests:
            print(f"  Rust Tests: {'PASSED' if rust_success else 'FAILED'}")
        print()
        
        if self.failed_tests == 0 and rust_success:
            print(f"{GREEN}✅ All tests passed!{NC}")
            return 0
        else:
            print(f"{RED}❌ Some tests failed!{NC}")
            return 1

def main():
    """Main entry point"""
    # Parse command line arguments
    args = sys.argv[1:]
    test_files = []
    cleanup = True
    db_prefix = "./test-db"
    use_temp = False
    run_rust_tests = False
    
    i = 0
    while i < len(args):
        if args[i] == "--no-cleanup":
            cleanup = False
        elif args[i] == "--testing":
            use_temp = True
        elif args[i] == "--rust":
            run_rust_tests = True
        elif args[i] == "--db-prefix":
            if i + 1 < len(args):
                db_prefix = args[i + 1]
                i += 1
            else:
                print("Error: --db-prefix requires a prefix")
                sys.exit(1)
        elif not args[i].startswith("--"):
            test_files.append(args[i])
        i += 1
    
    # Set up database directory
    if use_temp:
        import tempfile
        db_dir = tempfile.mkdtemp(prefix="echo-test-")
    else:
        import time
        db_dir = f"{db_prefix}-{int(time.time())}-{os.getpid()}"
    
    runner = EchoTestRunner(db_dir)
    
    # Run tests and exit with appropriate code
    exit_code = runner.run_all_tests(test_files if test_files else None, run_rust_tests=run_rust_tests)
    
    # Cleanup test database unless --no-cleanup was specified
    if cleanup:
        import shutil
        try:
            shutil.rmtree(runner.db_dir)
            print(f"\nCleaned up test database: {runner.db_dir}")
        except Exception as e:
            print(f"\nWarning: Failed to clean up test database: {e}")
    else:
        print(f"\nTest database preserved: {runner.db_dir}")
    
    sys.exit(exit_code)

if __name__ == "__main__":
    main()