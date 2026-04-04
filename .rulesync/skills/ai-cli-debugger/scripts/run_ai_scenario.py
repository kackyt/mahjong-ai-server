import sys
import subprocess
import argparse
import shlex

def main():
    parser = argparse.ArgumentParser(description="Run TUI AI Debug scenario")
    parser.add_argument("bin_cmd", help="Command to run the binary, e.g. 'cargo run -p openwars_cli --features ai-debug'")
    parser.add_argument("--keys", required=True, help="Space-separated list of keys, e.g. '5*right 3*down enter dump q'")
    args = parser.parse_args()

    # Expand keys
    expanded_keys = []
    for token in args.keys.split():
        if '*' in token:
            parts = token.split('*')
            if len(parts) == 2 and parts[0].isdigit() and parts[1]:
                expanded_keys.extend([parts[1]] * int(parts[0]))
            else:
                parser.error(f"Invalid key macro: '{token}'. Use N*key (e.g. 5*right).")
        else:
            expanded_keys.append(token)
    
    input_str = "\n".join(expanded_keys) + "\n"
    
    # Run process
    process = subprocess.Popen(
        shlex.split(args.bin_cmd),
        shell=False,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE, # Keep stderr separate to avoid noise in stdout dump
        text=True,
        encoding='utf-8'
    )
    
    try:
        stdout, stderr = process.communicate(input=input_str, timeout=30)
    except subprocess.TimeoutExpired:
        process.kill()
        stdout, stderr = process.communicate()
        print("--- Process timed out after 30s ---", file=sys.stderr)
        if stdout:
            print(stdout)
        if stderr:
            print("STDERR:", file=sys.stderr)
            print(stderr, file=sys.stderr)
        sys.exit(124)
    
    print(stdout)
    if process.returncode != 0:
        print(f"--- Process exited with code {process.returncode} ---", file=sys.stderr)
        if stderr:
            print("STDERR:", file=sys.stderr)
            print(stderr, file=sys.stderr, end="")
        sys.exit(process.returncode)

if __name__ == "__main__":
    main()
