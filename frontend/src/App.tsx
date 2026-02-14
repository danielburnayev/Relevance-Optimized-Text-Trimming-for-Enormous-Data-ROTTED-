import { useState } from 'react';
import TextFormElement from './TextFormElement';
import whiteZip from './assets/zip logo white.webp';
import blackZip from './assets/zip logo black.webp';
import './App.css';

const maxZipSize: number = 50000000;

function App() {
  const [currZipImg, setZipImg] = useState(whiteZip);

  return (
    // outermost <> is a div with id root
    <> 
      <header className="flex items-center justify-between border-b-2 border-white min-w-[98vw] min-h-[10vh] pl-2.5 pr-2.5">
        <div>
          <h1 className="text-4xl">ROTTED</h1>
          <h2>Relevance Optimized Text Trimming for Enormous Data</h2>
        </div>
        <button id="settings-btn">Settings</button>
      </header>

      <form id="query-form" className="flex flex-col items-center min-w-full h-[90vh]" onSubmit={submitToBackend}>
        <div className="flex items-center justify-center border-b-[0.75px] border-r-white w-[95%] h-[40%]">
          <TextFormElement givenID="desired-outcome-text-field" labelText="Desired Outcome: " placeholderText="Wow"/>
        </div>
        <div className="flex flex-col items-center justify-center w-[95%] h-[60%]">
          <h2>Accepts only one .zip file containing text files</h2>
          <h3>Less than or equal to 50GB</h3>
          
          <div id="give-zip-btn" className="flex flex-col items-center justify-center border-3 border-white h-1/2 aspect-square cursor-pointer mt-3.75 mb-1.5" 
               onClick={activateHiddenFileInput}
               onMouseOver={() => setZipImg(blackZip)}
               onMouseLeave={() => setZipImg(whiteZip)}> 
            <img src={currZipImg} />
          </div>
          <div id="zip-name-container" className="hidden w-1/3 max-w-1/2 mb-3.75">
            <p id="zip-name" className="mr-2.5"></p>
            <button className='flex items-center justify-center h-5.5 aspect-square' 
                    onClick={() => {hideObj("submit-btn"); hideObj("zip-name-container"); resetHiddenFileInput();}}>
                      X
            </button>
          </div>

          <input type="file" id="file-upload" accept=".zip" className="hidden" onChange={readyForSubmit}/> {/* Hidden file input that will get activated when zip-name-container is clicked. ALWAYS STAYS HIDDEN */}
          <input type="submit" value="Submit" id="submit-btn" className="hidden w-1/6 aspect-16/5 text-3xl"/>
        </div>
      </form>
    </>
  )
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

function readyForSubmit() {
  const fileUploadInput: HTMLInputElement | null = document.getElementById("file-upload") as HTMLInputElement;
  const zipName: HTMLParagraphElement | null = document.getElementById("zip-name") as HTMLParagraphElement;
  const zipNameContainer: HTMLElement | null = revealHiddenObj("zip-name-container");
  revealHiddenObj("submit-btn");

  if (zipNameContainer && fileUploadInput && fileUploadInput.files && fileUploadInput.files.length == 1 && fileUploadInput.files[0].size <= maxZipSize && zipName) {
    zipNameContainer.classList.add("flex");
    zipNameContainer.classList.add("flex-row");
    zipNameContainer.classList.add("justify-center");
    zipNameContainer.classList.add("items-center");
    zipName!.textContent = fileUploadInput.files[0].name;
  }
}

function submitToBackend(event: React.SyntheticEvent<HTMLFormElement>) {
  event.preventDefault();
}

export default App;
