import { useState } from 'react';
import TextFormElement from './TextFormElement';
import whiteZip from './assets/zip logo white.webp';
import blackZip from './assets/zip logo black.webp';
import './App.css';

const maxZipSize: number = 50000000;

function App() {
  const [currZipImg, setZipImg] = useState(whiteZip);
  const [showInstructions, setInstructions] = useState(true);
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [submitStatus, setSubmitStatus] = useState<{ type: 'success' | 'error' | null; message: string }>({ type: null, message: '' });

  return (
    // outermost <> is a div with id root
    <> 
      <header className="flex items-center justify-between border-b-2 border-white min-w-[98vw] min-h-[10vh] pl-2.5 pr-2.5">
        <div>
          <h1 className="text-4xl">ROTTED</h1>
          <h2>Relevance Optimized Text Trimming for Enormous Data</h2>
        </div>
        <button id="settings-btn" onClick={() => setSidebarOpen(!sidebarOpen)}>Settings</button>
      </header>

      <form id="query-form" className="flex flex-col items-center justify-around min-w-full h-[90vh]" onSubmit={submitToBackend}>
        {submitStatus.type && (
          <div className={`w-[96%] p-4 rounded mb-4 ${submitStatus.type === 'success' ? 'bg-[#1b1b1b] text-green-100' : 'bg-red-900 text-red-100'}`}>
            {submitStatus.message}
          </div>
        )}
        <div className="flex flex-col w-[96%] h-[27.5%] bg-[#1b1b1b]">
          {(showInstructions) ? <h1 className="text-5xl h-1/5 ml-2 mt-2">1: Specify what you want to look for in text files</h1> : <></>} 
          
          <div className={`flex flex-row items-center justify-center ${(showInstructions) ? "h-4/5" : "h-full"}`}>
            <TextFormElement givenID="desired-outcome-text-field" labelText="Desired Outcome: " placeholderText="Outcome"/>
          </div>
        </div>

        <div className="flex flex-col w-[96%] h-[70%] bg-[#1b1b1b]">
          {(showInstructions) ? <h1 className="text-5xl h-1/7 ml-2 mt-2">2: Provide package file of your text files</h1> : <></>} 

          <div className={`flex flex-col items-center ${(showInstructions) ? "h-6/7" : "h-full justify-center"}`}>
            <h2>Accepts only one .zip file containing text files</h2>
            <h3>Less than or equal to 50GB</h3>
            
            <div id="give-zip-btn" className="flex flex-col items-center justify-center border-3 border-white min-h-1/2 max-h-1/2 aspect-square cursor-pointer mt-3.75 mb-1.5" 
                onClick={activateHiddenFileInput}
                onMouseOver={() => setZipImg(blackZip)}
                onMouseLeave={() => setZipImg(whiteZip)}> 
              <img src={currZipImg} />
            </div>
            <div id="zip-name-container" className="hidden w-2/3 max-w-7/8 mb-3.75">
              <p id="zip-name" className="mr-2.5"></p>
              <button type="button" className='flex items-center justify-center h-5.5 aspect-square' 
                      onClick={() => {hideObj("submit-btn"); hideObj("zip-name-container"); resetHiddenFileInput();}}>
                        X
              </button>
            </div>

            <input type="file" id="file-upload" accept=".zip" className="hidden" onChange={prepForSendingOver}/> {/* Hidden file input that will get activated when zip-name-container is clicked. ALWAYS STAYS HIDDEN */}
            <input type="submit" value={isSubmitting ? "Processing..." : "Submit"} id="submit-btn" className="hidden min-w-1/6 max-w-1/4 aspect-16/5 text-3xl disabled:opacity-50 disabled:cursor-not-allowed" disabled={isSubmitting}/>
          </div>
        </div>
      </form>

      {/* Sidebar */}
      <div 
        className={`fixed top-0 right-0 h-screen w-[17.5%] min-w-[280px] bg-[#1b1b1b] border-l-2 border-white transition-transform duration-300 ease-in-out z-50 ${
          sidebarOpen ? 'translate-x-0' : 'translate-x-full'
        }`}
      >
        <div className="p-6">
          <div className="flex items-center justify-between mb-8">
            <h2 className="text-2xl font-bold">Settings</h2>
            <button 
              onClick={() => setSidebarOpen(false)}
              className="text-2xl hover:text-gray-400 transition-colors"
            >
              ×
            </button>
          </div>

          <div className="flex items-center justify-between py-4 border-b border-gray-700">
            <label htmlFor="instructions-toggle" className="text-lg cursor-pointer">
              Show Instructions
            </label>
            <button
              id="instructions-toggle"
              onClick={() => setInstructions(!showInstructions)}
              className={`relative w-12 h-6 sm:w-14 sm:h-7 rounded-full transition-colors duration-300 flex-shrink-0 ${
                showInstructions ? 'bg-blue-600' : 'bg-gray-600'
              }`}
            >
              <span
                className={`absolute top-0.5 left-0.5 w-5 h-5 sm:w-6 sm:h-6 bg-white rounded-full transition-transform duration-300 ${
                  showInstructions ? 'translate-x-6 sm:translate-x-7' : 'translate-x-0'
                }`}
              />
            </button>
          </div>
        </div>
      </div>

      {/* Overlay */}
      {sidebarOpen && (
        <div 
          className="fixed inset-0 bg-[#1b1b1b] opacity-50 z-40"
          onClick={() => setSidebarOpen(false)}
        />
      )}
    </>
  );
  
  async function submitToBackend(event: React.SyntheticEvent<HTMLFormElement>) {
    event.preventDefault();

    const fileUploadInput: HTMLInputElement | null = document.getElementById("file-upload") as HTMLInputElement;
    const desiredOutcomeInput: HTMLInputElement | null = document.getElementById("desired-outcome-text-field") as HTMLInputElement;
    let errorOccured: boolean = false;

    if (!fileUploadInput || !fileUploadInput.files || fileUploadInput.files.length == 0) {
      setMissingFieldFlash(document.getElementById("give-zip-btn"));
      console.error("No file selected");
      errorOccured = true;
    }
    if (!desiredOutcomeInput || !desiredOutcomeInput.value.trim()) {
      setMissingFieldFlash(document.getElementById("desired-outcome-text-field-container"));
      console.error("No desired outcome entered");
      errorOccured = true;
    }

    if (errorOccured) {return;}
    
    const zipFile = fileUploadInput!.files![0];
    const desiredOutcome = desiredOutcomeInput.value;
    
    setIsSubmitting(true);
    setSubmitStatus({ type: null, message: '' });
    
    // Convert file to base64
    const reader = new FileReader();
    reader.onload = async function(e) {
      const base64File = e.target?.result as string;
      
      const dataToSend = {
        desiredOutcome: desiredOutcome,
        zipFile: base64File,
        fileName: zipFile.name,
        fileSize: zipFile.size
      };
      
      try {
        const response = await fetch("http://127.0.0.1:5000", {
          method: "POST",
          headers: {
            "Content-Type": "application/json"
          },
          body: JSON.stringify(dataToSend)
        });
        
        if (response.ok) {
          const result = await response.json();
          console.log("Success:", result);
          setSubmitStatus({ type: 'success', message: 'File processed successfully! Check the console for results.' });
          // Optional: Reset form after success
          setTimeout(() => {
            resetForm();
          }, 2000);
        } else {
          console.error("Error:", response.statusText);
          setSubmitStatus({ type: 'error', message: `Error: ${response.statusText}. Please try again.` });
        }
      } 
      catch (error) {
        console.error("Fetch error:", error);
        setSubmitStatus({ type: 'error', message: `Connection error: ${error instanceof Error ? error.message : 'Unknown error'}. Make sure the backend is running.` });
      }
      finally {
        setIsSubmitting(false);
      }
    };
    
    reader.readAsDataURL(zipFile);
  }
}

function activateHiddenFileInput() {
  const hiddenFileInput: HTMLElement | null = document.getElementById("file-upload");
  if (hiddenFileInput) {hiddenFileInput.click();}
}

function resetHiddenFileInput() {
  const hiddenFileInput: HTMLInputElement | null = document.getElementById("file-upload") as HTMLInputElement;
  if (hiddenFileInput) {hiddenFileInput.value = '';}
}

function hideObj(objID: string) {
  const theObj: HTMLElement | null = document.getElementById(objID);
  if (theObj) {
    theObj.classList.remove("flex");
    theObj.classList.add("hidden");
  }
}

function revealHiddenObj(objID: string): HTMLElement | null {
  const theObj: HTMLElement | null = document.getElementById(objID);
  if (theObj) {
    theObj.classList.remove("hidden");
    return theObj;
  }
  return null;
}

function prepForSendingOver() {
  const fileUploadInput: HTMLInputElement | null = document.getElementById("file-upload") as HTMLInputElement;
  const zipName: HTMLParagraphElement | null = document.getElementById("zip-name") as HTMLParagraphElement;
  const zipNameContainer: HTMLElement | null = revealHiddenObj("zip-name-container");
  revealHiddenObj("submit-btn");

  if (zipNameContainer && fileUploadInput?.files && fileUploadInput.files.length === 1 && fileUploadInput.files[0].size <= maxZipSize && zipName) {
    zipNameContainer.classList.add("flex", "flex-row", "justify-center", "items-center");
    zipName.textContent = fileUploadInput.files[0].name;
  }
}

function setMissingFieldFlash(field: HTMLElement | null) {
  if (field) {
    field!.classList.add("missing-field-flash");
    setTimeout(() => {field!.classList.remove("missing-field-flash");}, 1500);
  }
}

function resetForm() {
  hideObj("submit-btn");
  hideObj("zip-name-container");
  resetHiddenFileInput();
  const outcomeInput = document.getElementById("desired-outcome-text-field") as HTMLInputElement;
  if (outcomeInput) {
    outcomeInput.value = '';
  }
}

export default App;