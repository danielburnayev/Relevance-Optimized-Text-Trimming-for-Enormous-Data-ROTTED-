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
os.makedirs(UPLOAD_DIR,exist_ok=True)
os.makedirs(EXTRACT_DIR,exist_ok=True)
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
   body =request.get_json()
   print(body)


   userinput=body["desiredOutcome"]
   kw_model = KeyBERT()
   keywords=kw_model.extract_keywords(userinput)
   b64 = body["zipFile"]
   filename = body["fileName"]
   file_bytes = base64.b64decode(b64)
   upload_path= os.path.join(UPLOAD_DIR,filename)
   
   with open(upload_path, "wb") as f:
        f.write(file_bytes) 
   
   with zipfile.ZipFile(upload_path, "r") as z:
        z.extractall(EXTRACT_DIR)

   return jsonify({"ok": True, "saved": EXTRACT_DIR, "extracted_to": EXTRACT_DIR})

if __name__ == '__main__':
    app.run(debug=True)





