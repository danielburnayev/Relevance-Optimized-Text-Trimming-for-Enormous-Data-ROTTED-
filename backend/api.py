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
      print("Received body:", body)

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
          with zipfile.ZipFile(upload_path, "r") as z:
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

  out_file = os.path.join(FINALE_DIR, f"results_{now}.txt")
  count = 0
  with open(out_file, "w", encoding="utf-8") as out:
       for root, dirs, files in os.walk(EXTRACT_DIR):
           dirs[:] = [d for d in dirs if d != "__MACOSX"]


           for file_name in files:
               if file_name == ".DS_Store":
                   continue


               path = os.path.join(root, file_name)


               with open(path, "r", encoding="utf-8", errors="ignore") as f:
                   text = f.read()


               if not text:
                   continue
              
               keywords = [kw for kw, _ in kw_model.extract_keywords(userinput)]


               k = max(1, int(len(text) * 0.025))


               ranges = []
               for kw in keywords:
                   pos = 0
                   while True:
                       start = text.find(kw, pos)
                       if start == -1:
                           break


                       end = start + len(kw)
                       a = max(0, start - k)
                       b = min(len(text), end + k)
                       ranges.append((a, b))
                       pos = end


               if not ranges:
                   continue


               ranges.sort()
               merged = []
               for a, b in ranges:
                   if not merged or a > merged[-1][1]:
                       merged.append([a, b])
                   else:
                       merged[-1][1] = max(merged[-1][1], b)


               for a, b in merged:
                   out.write(text[a:b])
                   out.write("\n\n")

               
               result=','.join(keywords)
               out.write(result)

               count += len(merged)

  print(f"Processing complete. Total ranges: {count}")
  return {"count": count, "output_file": out_file}
 




  
if __name__ == '__main__':
   app.run(debug=True)


