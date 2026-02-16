import io
from flask import Flask
import os
import json,re
from flask import request
from flask import flash,redirect,jsonify
import zipfile,os,base64
from keybert import KeyBERT
import random
import time
from datetime import datetime
from flask_cors import CORS


app = Flask(__name__)
CORS(app)
app.config['CORS_HEADERS'] = 'Content-Type'


@app.route('/') #just testing how flask worked
def home():
   return '<h1>Flask REST starting guys</h1>'


now=datetime.now().strftime("%Y%m%d_%H%M")
UPLOAD_DIR = 'UPLOAD'
EXTRACT_DIR = 'EXTRACT'+now
FINALE_DIR = "FINALE"+now
os.makedirs(UPLOAD_DIR,exist_ok=True)
os.makedirs(EXTRACT_DIR,exist_ok=True)
os.makedirs(FINALE_DIR, exist_ok=True)
app.config['FINALE_DIR']=FINALE_DIR
app.config['UPLOAD_DIR'] = UPLOAD_DIR
app.config['EXTRACT_DIR'] = EXTRACT_DIR
#const dataToSend = {
    # desiredOutcome: desiredOutcome,
    # zipFile: base64File,
    # fileName: zipFile.name,
    # fileSize: zipFile.size
   #};


@app.post('/userinput')
def userinput():
  try:
      body = request.get_json()
      if not body:
          return {"error": "Empty request body"}, 400

      # Validate required fields
      if "zipFile" not in body:
          return {"error": "Missing zipFile in request"}, 400
      if "fileName" not in body:
          return {"error": "Missing fileName in request"}, 400
      if "usedNaturalLanguage" not in body:
          return {"error": "Missing usedNaturalLanguage in request"}, 400

      usedNaturalLanguage = body["usedNaturalLanguage"]
      
      if not usedNaturalLanguage:
          if "fields" not in body:
              return {"error": "Missing fields in request"}, 400
          fields = body["fields"]
          
          # Extract desired outcome as the primary search term
          desiredOutcome = fields.get("desiredOutcome", "").strip()
          if not desiredOutcome:
              return {"error": "desiredOutcome cannot be empty"}, 400
          
          # Build userinput with available fields
          userinput_parts = [desiredOutcome]
          if fields.get("date"):
              userinput_parts.append(fields.get("date"))
          if fields.get("people"):
              userinput_parts.append(fields.get("people"))
          if fields.get("events"):
              userinput_parts.append(fields.get("events"))
          if fields.get("location"):
              userinput_parts.append(fields.get("location"))
          
          userinput = " ".join(userinput_parts)
      else:
          if "naturalLanguageDescription" not in body:
              return {"error": "Missing naturalLanguageDescription in request"}, 400
          naturalLanguageDescription = body["naturalLanguageDescription"].strip()
          if not naturalLanguageDescription:
              return {"error": "naturalLanguageDescription cannot be empty"}, 400
          userinput = naturalLanguageDescription

      print(f"Using input: {userinput}")
      
      kw_model = KeyBERT()
      b64 = body["zipFile"]
      filename = body["fileName"]
      
      # Handle data URLs with base64 prefix (e.g., data:application/zip;base64,...)
      if b64.startswith("data:"):
          b64 = b64.split(",", 1)[1]
      
      try:
          file_bytes = base64.b64decode(b64)
      except Exception as e:
          print(f"Base64 decode error: {str(e)}")
          return {"error": f"Failed to decode base64: {str(e)}"}, 400
      
      upload_path = os.path.join(UPLOAD_DIR, filename)
      print(f"Upload path: {upload_path}, File size: {len(file_bytes)} bytes")
   
      with open(upload_path, "wb") as f:
           f.write(file_bytes)
   
      try:
          with zipfile.ZipFile(io.BytesIO(file_bytes), "r") as z:
               z.extractall(EXTRACT_DIR)
      except zipfile.BadZipFile as e:
          print(f"Zip file error: {str(e)}")
          return {"error": f"Invalid zip file: {str(e)}"}, 400
      except Exception as e:
          print(f"Unexpected zip error: {str(e)}")
          return {"error": f"Error extracting zip: {str(e)}"}, 400

  except Exception as e:
      print(f"Unexpected error in userinput: {str(e)}")
      return {"error": f"Unexpected error: {str(e)}"}, 500

  count = 0

  with zipfile.ZipFile("results.zip", "w", compression=zipfile.ZIP_DEFLATED) as z:
    for root, dirs, files in os.walk(EXTRACT_DIR):
      dirs[:] = [d for d in dirs if d != "__MACOSX"]

      for file_name in files:
          if file_name == ".DS_Store":
              continue

          path = os.path.join(root, file_name)
          z.write(path, arcname=file_name)
          count += 1

  try:
    # Read the zip file and encode it to base64
    with open("results.zip", "rb") as f:
      raw_file_data = f.read()
      base64_encoded_bytes = base64.b64encode(raw_file_data)
      base64_encoded_string = base64_encoded_bytes.decode('utf-8')
      print(f"Successfully encoded zip file ({len(raw_file_data)} bytes)")
  except Exception as e:
    print(f"Error reading zip file for base64 encoding: {str(e)}")
    return {"error": f"Failed to read zip file: {str(e)}"}, 500

  return {"count": count, "output_file": base64_encoded_string}
 

if __name__ == '__main__':
   app.run(debug=True)

