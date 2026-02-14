import './App.css';
import TextFormElement from './TextFormElement';

function App() {
  return (
    // outermost <> is a div with id root
    <> 
      <header className="flex items-center justify-between border-b-2 border-white min-w-[98vw] min-h-[10vh] pl-2.5 pr-2.5">
        <h1>ROTTEN</h1>
        <button id="settings-btn">Settings</button>
      </header>

      <form id="query-form" className="flex flex-col items-center min-w-full h-[90vh]">
        <div className="flex items-center justify-center border-b-[0.75px] border-r-white w-[95%] h-[40%]">
          <TextFormElement givenID="desired-outcome-text-field" labelText="Desired Outcome: " placeholderText="Wow"/>
        </div>
        <div className="flex flex-col items-center justify-center w-[95%] h-[60%]">
          <h2>Accepts only one .zip file containing text files</h2>
          <h3>Less than or equal to 20GB</h3>
          <input type="file" id="file-upload" name="filename" accept=".zip" className="cursor-pointer border border-white"/>
        </div>
      </form>
      
    </>
  )
}

export default App;
