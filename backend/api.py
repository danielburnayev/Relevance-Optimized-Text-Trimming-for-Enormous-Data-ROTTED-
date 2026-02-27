import io
import os
import csv
import traceback
import base64, zipfile
from flask import Flask, request, jsonify
from flask_cors import CORS
from query_handle import process_queries

app = Flask(__name__)
CORS(app)
app.config['CORS_HEADERS'] = 'Content-Type'

UPLOAD_DIR = 'UPLOAD'
DOWNLOAD_DIR = 'DOWNLOAD'
os.makedirs(UPLOAD_DIR,exist_ok=True)
os.makedirs(DOWNLOAD_DIR,exist_ok=True)
app.config['UPLOAD_DIR'] = UPLOAD_DIR
app.config['DOWNLOAD_DIR'] = UPLOAD_DIR
RUST_ENDPOINT = os.getenv('RUST_ENDPOINT')

@app.route('/')
def home():
  return '<h1>Flask REST starting guys</h1>'

@app.post('/userinput')
def userinput():
  if not request.is_json:
    return jsonify({"error": "Content-Type must be application/json"}), 400

  data = request.json
  base64_zip = data.get('zipFile', '')
  zip_bytes = base64.b64decode(base64_zip)
  data_csv_path = os.path.join(app.config['UPLOAD_DIR'], 'converted_input_data.csv')
  
  with zipfile.ZipFile(io.BytesIO(zip_bytes), 'r') as z:
    with open(data_csv_path, 'w', newline='', encoding='utf-8-sig') as f:
      writer = csv.writer(f)    
      writer.writerow(['content']) 

      for file_info in z.infolist():
        if file_info.is_dir() or file_info.filename.startswith('__MACOSX'):
          continue

        filename = file_info.filename
        if file_info.filename.endswith(('.txt', '.json', '.csv')):
          raw_bytes = z.read(filename)
          text_data = raw_bytes.decode('utf-8', errors='replace')
          extracted_contents = []

        if file_info.filename.endswith('.csv'):
          f_io = io.StringIO(text_data)
          reader = csv.reader(f_io)
          try:
            next(reader)
            first_data_row = next(reader)
            longest_idx = max(range(len(first_data_row)), key=lambda i: len(str(first_data_row[i])))
            extracted_contents.append(str(first_data_row[longest_idx]))

            for row in reader:
              if len(row) > longest_idx:
                extracted_contents.append(str(row[longest_idx]))
          except StopIteration:
            pass

        elif file_info.filename.endswith('.json'):
          try:
            json_data = json.loads(text_data)
                  
            if isinstance(json_data, list) and len(json_data) > 0 and isinstance(json_data[0], dict):
              first_obj = json_data[0]
              longest_key = max(first_obj.keys(), key=lambda k: len(str(first_obj.get(k, ''))))
                    
              for obj in json_data:
                if longest_key in obj:
                  extracted_contents.append(str(obj[longest_key]))
                        
            elif isinstance(json_data, dict):
              longest_key = max(json_data.keys(), key=lambda k: len(str(json_data.get(k, ''))))
              extracted_contents.append(str(json_data[longest_key]))
          except json.JSONDecodeError:
            pass

        elif file_info.filename.endswith('.txt'):
          lines = text_data.splitlines()
          for line in lines:
            clean_line = line.strip()
            if clean_line:
              extracted_contents.append(clean_line)

        for content in extracted_contents:
          clean_content = str(content).replace('\n', ' ').replace('\r', ' ').replace(',', ' ')
          clean_content = ' '.join(clean_content.split())
          writer.writerow([filename, clean_content])

  try:
    output_zip_path = process_queries(
      queries=data, 
      data_csv_path=data_csv_path, 
      target_col_index=1, 
      endpoint_url=RUST_ENDPOINT
    )
        
    with open(output_zip_path, 'rb') as f:
      out_base64 = base64.b64encode(f.read()).decode('utf-8')
            
    with zipfile.ZipFile(output_zip_path, 'r') as z:
      file_count = len([name for name in z.namelist() if not name.endswith('/')])
            
    return jsonify({
      "output_file": out_base64,
      "count": file_count
    }), 200
        
  except Exception as e:
    traceback.print_exc()
    return jsonify({"error": str(e)}), 500

if __name__ == '__main__':
  app.run(debug=True)
