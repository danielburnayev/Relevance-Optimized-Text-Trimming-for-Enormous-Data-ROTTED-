import zipfile
import sys
import os

def unzip_file(zip_src, dest_dir):
  """
  Unzips a file to a specified directory.
  Creates the directory if it doesn't exist.
  """
  # Create the destination folder if it's missing
  if not os.path.exists(dest_dir):
    os.makedirs(dest_dir)

    with zipfile.ZipFile(zip_src, 'r') as zip_ref:
      zip_ref.extractall(dest_dir)
      print(f"Successfully extracted to: {dest_dir}")

def main():
  if len(sys.argv) > 1 and len(sys.argv) < 4:
    zip_src = sys.argv[1]
    dest_dir = sys.argv[2]

    print("Extracting zip file...")
    unzip_file(zip_src, dest_dir)

  else:
    print("Incorrect number of arguments")
    for s in sys.argv:
      print(s)

main()
