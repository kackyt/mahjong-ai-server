import subprocess
import sys
import json

def run_command(args):
    cmd = ["pnpm", "jules"] + args
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        return result.stdout
    except subprocess.CalledProcessError as e:
        print(f"Error running command: {e.stderr}", file=sys.stderr)
        return None

def list_sessions():
    output = run_command(["remote", "list", "--session"])
    if output:
        print(output)

def create_session(prompt):
    output = run_command(["remote", "new", "--session", prompt])
    if output:
        print(output)

def pull_session(session_id, apply=False):
    args = ["remote", "pull", "--session", session_id]
    if apply:
        args.append("--apply")
    output = run_command(args)
    if output:
        print(output)

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python jules_helper.py [list|new|pull] [args...]")
        sys.exit(1)
    
    cmd = sys.argv[1]
    if cmd == "list":
        list_sessions()
    elif cmd == "new" and len(sys.argv) > 2:
        create_session(sys.argv[2])
    elif cmd == "pull" and len(sys.argv) > 2:
        apply = "--apply" in sys.argv
        session_id = sys.argv[2]
        pull_session(session_id, apply)
    else:
        print("Invalid arguments")
