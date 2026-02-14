from flask import Flask
import os
from flask import request
from flask import flash,redirect
import zipfile

app = Flask(__name__)
UPLOAD_DIR = '/Users/aaronpinto/Documents'
os.makedirs(UPLOAD_DIR,exist_ok=True)
ALLOWED_EXTENSIONS={'txt','pdf','zip'}
app.config['UPLOAD_DIR'] = UPLOAD_DIR

@app.route('/') #just testing how flask worked
def home():
    return '<h1>Flask REST starting guys</h1>'

if __name__ == '__main__':
    app.run(debug=True)
def allowed_file(filename): 
    return '.' in filename and filename.rsplit('.',1)[1].lower() in ALLOWED_EXTENSIONS

@app.post('/upload')
def uploadomega():
    if 'file' not in request.files:
        flash('no file part')
        return redirect(request.url)
    file =request.files['file']
    if file.filename == "":
        flash("No Selected files")
        return redirect(request.url)
    if file and allowed_file(file.filename):
        # everything under this is the stack overflow answer please fill in results
        zipfile_ob = zipfile.ZipFile(file_like_object)
    file_names = zipfile_ob.namelist
    # Filter names to only include the filetype that you want:
    file_names = [file_name for file_name in file_names if file_name.endswith(".txt")]
    files = [(zipfile_ob.open(name).read(),name) for name in file_names]
    return str(files)

