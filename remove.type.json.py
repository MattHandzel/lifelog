# This script walks through the directory and deletees all files that end with ".type.json"

import os


def remove_type_json_files(directory):
    for root, dirs, files in os.walk(directory):
        for file in files:
            if file.endswith(".type.json"):
                file_path = os.path.join(root, file)
                try:
                    os.remove(file_path)
                    print(f"Removed: {file_path}")
                except Exception as e:
                    print(f"Error removing {file_path}: {e}")


remove_type_json_files("./")
