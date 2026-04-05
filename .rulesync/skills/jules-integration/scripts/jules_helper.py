import argparse

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
    parser = argparse.ArgumentParser(description="Jules helper script")
    subparsers = parser.add_subparsers(dest="command", help="Commands")

    # List command
    subparsers.add_parser("list", help="List remote sessions")

    # New command
    new_parser = subparsers.add_parser("new", help="Create a new remote session")
    new_parser.add_argument("prompt", help="Instructions for the new session")

    # Pull command
    pull_parser = subparsers.add_parser("pull", help="Pull results from a remote session")
    pull_parser.add_argument("session_id", help="ID of the session to pull")
    pull_parser.add_argument("--apply", action="store_true", help="Apply the pulled changes")

    args = parser.parse_args()

    if args.command == "list":
        list_sessions()
    elif args.command == "new":
        create_session(args.prompt)
    elif args.command == "pull":
        pull_session(args.session_id, args.apply)
    else:
        parser.print_help()
