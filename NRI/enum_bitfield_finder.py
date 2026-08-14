import os
import re
import sys


def extract_base_names(file_path):
    # Check if the file actually exists before opening it
    if not os.path.isfile(file_path):
        print(f"Error: The file '{file_path}' does not exist.")
        sys.exit(1)

    pattern = re.compile(r"^pub\s+const\s+([a-zA-Z0-9]+)_[a-zA-Z0-9_]+")
    seen_names = set()

    with open(file_path, "r", encoding="utf-8") as file:
        for line in file:
            match = pattern.match(line.strip())
            if match:
                base_name = match.group(1)
                if base_name not in seen_names:
                    print(base_name)
                    seen_names.add(base_name)


if __name__ == "__main__":
    # Ensure the user provided a file argument
    if len(sys.argv) < 2:
        print("Usage: python script.py <filename>")
        sys.exit(1)

    # Get the filename from the first command-line argument
    target_file = sys.argv[1]
    extract_base_names(target_file)
