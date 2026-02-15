import { useState } from 'react';
import TextFormElement from './TextFormElement';
import whiteZip from './assets/zip logo white.webp';
import blackZip from './assets/zip logo black.webp';
import downloadIcon from './assets/download logo.webp';
import './App.css';

interface ZipInfo {
    base64Encoding: string;
    zipSize: number;
    numberOfFiles: number;
}

const maxZipSize: number = 50000000;

function App() {
  const [currZipImg, setZipImg] = useState(whiteZip);
  const [showInstructions, setInstructions] = useState(true);
  const [useNaturalLanguage, setUseNaturalLanguage] = useState(false);
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [submitStatus, setSubmitStatus] = useState<{ type: 'success' | 'error' | null; message: string }>({ type: null, message: '' });
  const [receivedZip, setReceivedZip] = useState<ZipInfo | null>({ base64Encoding: 'dataapplication/zipbase64UEsDBBQACAAIAGGMTlwAAAAAAAAAAAAAAAAIACAAdGVzdC50eHR1eAsAAQT1AQAABBQAAABVVA0AB5f4kGmZ+JBpl/iQaTWIMQoAMBDCHnavEQ4HdfP/UFroEpKIq2KdSzrgk46cJRqPiK0zuv3/AVBLBwjlLFRHKgAAADgAAABQSwMEFAAIAAgAYYxOXAAAAAAAAAAAAAAAABMAIABfX01BQ09TWC8uX3Rlc3QudHh0dXgLAAEE9QEAAAQUAAAAVVQNAAeX+JBpmfiQaZ74kGljYBVjZ2BiYPBNTFbwD1aIUIACkBgDJxAbAXEhEIP4ixmIAo4hIUFQJkjHDCDmRlPCiBAXTc7P1UssKMhJ1Ssoyi9LzUvMS05lYGRieOSvm6d6p+8uAFBLBwjthPYGVgAAAKMAAABQSwECFAMUAAgACABhjE5c5SxURyoAAAA4AAAACAAYAAAAAAAAAAAApIEAAAAAdGVzdC50eHR1eAsAAQT1AQAABBQAAABVVAUAAZf4kGlQSwECFAMUAAgACABhjE5c7YT2BlYAAACjAAAAEwAYAAAAAAAAAAAApIGAAAAAX19NQUNPU1gvLl90ZXN0LnR4dHV4CwABBPUBAAAEFAAAAFVUBQABl/iQaVBLBQYAAAAAAgACAKcAAAA3AQAAAAA', zipSize: 123456765, numberOfFiles: 5 });

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
        <div className="flex flex-col justify-around w-[96%] h-[25%] bg-[#1b1b1b]">
          {(showInstructions) && 
            <h1 className="text-4xl h-1/5 ml-2 mt-2 mb-[5px]">1: Specify what you want to look for in text files</h1>
          } 
          
          {(!useNaturalLanguage) ? 
            <div className={`flex flex-row items-center justify-between m-auto w-9/10 ${(showInstructions) ? "h-4/5" : "h-full"}`}>
              <TextFormElement givenID="desired-outcome-text-field" labelText="Outcome: " placeholderText="Ex: Burglary"/>
              <TextFormElement givenID="important-date-field" labelText="Date: " placeholderText="Ex: 12/2/23, 12/2/2023"/>
              <TextFormElement givenID="important-people-field" labelText="People: " placeholderText="Ex: Jane Smith"/>
              <TextFormElement givenID="important-events-field" labelText="Events: " placeholderText="Ex: Went to London"/>
              <TextFormElement givenID="important-location-field" labelText="Location: " placeholderText="Ex: London, New York"/>
            </div>
            : 
            <textarea id="natural-language-input" rows={5} cols={50} placeholder="What you're specifically looking for in the text files. Feel free to describe this in any way you please, with as many details possible, given that it makes sense." 
                      className={`w-full h-full p-1.5 resize-none ${(showInstructions) ? "h-4/5 border-t-[0.5px] border-white" : "h-full"}`}/>
          } 
        </div>

        <div className="relative flex flex-row justify-between w-[96%] h-[70%]">
          {submitStatus.type && 
            setTimeout(() => setSubmitStatus({ type: null, message: '' }), 3000) &&
            (<div className={`absolute flex items-center justify-center w-[35%] min-h-[15%] top-[calc(50%-7.5%)] right-5 text-center cursor-pointer ${submitStatus.type === 'success' ? 'bg-[#1b1b1b] text-green-100' : 'bg-red-900 text-red-100'}`}
                  onClick={() => setSubmitStatus({ type: null, message: '' })}>
              {submitStatus.message}
            </div>
            )
          }

          <div className={`flex flex-col items-center bg-[#1b1b1b] h-full ${(!receivedZip) ? "w-full" : "w-[59.5%]"} ${(!showInstructions) ? "justify-center" : "" }`}>
             {(showInstructions) ? <h1 className="text-4xl h-1/7 mr-auto ml-2 mt-2">2: Provide package file of your text files</h1> : <></>} 
            
            <h2 className="text-center">
              1 .zip file {"<"}= 50GB <br/>
              Only .txt or .json files inside will be analyzed
            </h2>
            
            <div id="give-zip-btn" className="flex flex-col items-center justify-center border-3 border-white min-h-[45%] max-h-[45%] aspect-square cursor-pointer mt-3.75 mb-1.5" 
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

            {isSubmitting && (
              <div className="absolute inset-0 bg-[#000000aa] flex items-center justify-center z-40">
                <div className="flex flex-col items-center gap-4">
                  <div className="w-16 h-16 border-4 border-gray-600 border-t-blue-500 rounded-full animate-spin"></div>
                  <p className="text-xl text-gray-300">Processing your file...</p>
                </div>
              </div>
            )}
          </div>

          {receivedZip && (
            <div className="flex flex-col items-center bg-[#383838] h-full w-[39.5%]">
              <div className="flex items-center justify-between h-1/8 mb-[12.5%] w-full">
                <h1 className="text-4xl mr-auto ml-2 mt-2">{(showInstructions) ? "3: Download relevant files" : ""}</h1>
                <button className="mr-2 mt-2 h-[30px] aspect-square text-center" 
                        onClick={() => setReceivedZip(null)}>
                          X
                </button>
              </div>
              <a href={receivedZip.base64Encoding} download="results.zip" 
                 id="download-zip-btn"
                 className="flex flex-col items-center justify-center h-1/2 aspect-square px-6 py-3 text-black border-4 border-white bg-white"
                 onClick={() => setReceivedZip(null)}>

                <img src={downloadIcon}/>
                <p>{`results.zip (${(receivedZip.zipSize / (1024 * 1024)).toFixed(2)} MB)`}</p>
                <p>{`${receivedZip.numberOfFiles} files`}</p>
              </a>
            </div>
          )}

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
              className={`relative w-12 h-6 sm:w-14 sm:h-7 transition-colors duration-300 shrink-0 ${
                showInstructions ? 'bg-blue-600' : 'bg-gray-600'
              }`}
            >
              <span
                className={`absolute top-0 left-0 w-[26px] aspect-square bg-white transition-transform duration-300 ${
                  showInstructions ? 'translate-x-6 sm:translate-x-7' : 'translate-x-0'
                }`}
              />
            </button>
          </div>

          <div className="flex items-center justify-between py-4 border-b border-gray-700">
            <label htmlFor="natural-language-toggle" className="text-lg cursor-pointer">
              Describe With Natural Language
            </label>
            <button
              id="natural-language-toggle"
              onClick={() => setUseNaturalLanguage(!useNaturalLanguage)}
              className={`relative w-12 h-6 sm:w-14 sm:h-7 transition-colors duration-300 shrink-0 ${
                useNaturalLanguage ? 'bg-blue-600' : 'bg-gray-600'
              }`}
            >
              <span
                className={`absolute top-0 left-0 w-[26px] aspect-square  bg-white transition-transform duration-300 ${
                  useNaturalLanguage ? 'translate-x-6 sm:translate-x-7' : 'translate-x-0'
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
    function checkInput(obj: HTMLInputElement | HTMLTextAreaElement, containerID: string, errorMessage: string) {
      if (!obj || !obj.value.trim()) {checkInputWrapper(containerID, errorMessage);}
    }
    
    function checkInputWrapper(containerID: string, errorMessage: string) {
      setMissingFieldFlash(document.getElementById(containerID));
      finalErrorMessage += `${errorMessage}, `;
      errorOccured = true;
    }
    
    event.preventDefault();
    setSubmitStatus({ type: null, message: '' });

    const fileUploadInput: HTMLInputElement | null = document.getElementById("file-upload") as HTMLInputElement;
    const desiredOutcomeInput: HTMLInputElement | null = document.getElementById("desired-outcome-text-field") as HTMLInputElement;
    const dateInput: HTMLInputElement | null = document.getElementById("important-date-field") as HTMLInputElement;
    const peopleInput: HTMLInputElement | null = document.getElementById("important-people-field") as HTMLInputElement;
    const eventsInput: HTMLInputElement | null = document.getElementById("important-events-field") as HTMLInputElement;
    const locationInput: HTMLInputElement | null = document.getElementById("important-location-field") as HTMLInputElement;
    const naturalLanguageInput: HTMLTextAreaElement | null = document.getElementById("natural-language-input") as HTMLTextAreaElement;
    let errorOccured: boolean = false;
    let finalErrorMessage: string = "";

    checkInput(fileUploadInput, "give-zip-btn", "Input Error: Please select a .zip file to upload.");
    if (!useNaturalLanguage) {
      checkInput(desiredOutcomeInput, "desired-outcome-text-field-container", "Input Error: Please enter a desired outcome.");
      checkInput(dateInput, "important-date-field-container", "Input Error: Please enter an important date.");
      checkInput(peopleInput, "important-people-field-container", "Input Error: Please enter important people.");
      checkInput(eventsInput, "important-events-field-container", "Input Error: Please enter important events.");
      checkInput(locationInput, "important-location-field-container", "Input Error: Please enter an important location.");
    }
    else {
      checkInput(naturalLanguageInput, "natural-language-input", "Input Error: Please enter a natural language description.");
    }

    if (errorOccured) {
      setSubmitStatus({ type: 'error', message: finalErrorMessage });
      console.log(finalErrorMessage);
      setTimeout(() => setSubmitStatus({ type: null, message: '' }), 3000);
      return;
    }
    
    const zipFile = fileUploadInput!.files![0];
    const desiredOutcome = desiredOutcomeInput?.value;
    const importantDate = dateInput?.value;
    const importantPeople = peopleInput?.value;
    const importantEvents = eventsInput?.value;
    const importantLocation = locationInput?.value;
    const naturalLanguageDescription = naturalLanguageInput?.value;
    
    setIsSubmitting(true);
    setSubmitStatus({ type: null, message: '' });
    
    // Convert file to base64
    const reader = new FileReader();
    reader.onload = async function(e) {
      const base64File = e.target?.result as string;
      console.log(base64File);

      const portion = "data:application/zip;base64,";
      console.log(base64File.substring(portion.length));
      const base64WithoutPrefix = base64File.substring(portion.length);
      
      const fields = {
        desiredOutcome: desiredOutcome,
        importantDate: importantDate,
        importantPeople: importantPeople,
        importantEvents: importantEvents,
        importantLocation: importantLocation
      }

      const dataToSend = {
        usedNaturalLanguage: useNaturalLanguage,
        naturalLanguageDescription: naturalLanguageDescription,
        fields: fields,
        zipFile: base64WithoutPrefix,
        fileName: zipFile.name,
        fileSize: zipFile.size
      };
      
      try {
        const response = await fetch("http://127.0.0.1:5000/userinput", {
          method: "POST",
          headers: {
            "Content-Type": "application/json"
          },
          body: JSON.stringify(dataToSend)
        });
        
        if (response.ok) {
          const result = await response.json();
          console.log(result);

          const fullBase64Data = "data:application/zip;base64," + result.output_file;
          const zipFileSize = fullBase64Data.length * (3/4) - (fullBase64Data.endsWith("==") ? 2 : fullBase64Data.endsWith("=") ? 1 : 0);
          const newNumberOfFiles = result.count;
          console.log("Success:", result);
          setSubmitStatus({ type: 'success', message: 'File processed successfully! Check the console for results.' });
          setReceivedZip({ base64Encoding: fullBase64Data, 
                           zipSize: zipFileSize, 
                           numberOfFiles: newNumberOfFiles 
                         });
        } 
        else {
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
        resetForm();
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
    setTimeout(() => {field!.classList.remove("missing-field-flash");}, 3000);
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