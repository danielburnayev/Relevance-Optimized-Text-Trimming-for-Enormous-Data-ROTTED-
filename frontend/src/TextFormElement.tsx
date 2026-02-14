import { useState } from 'react';

interface GivenProps {
    givenID: string;
    labelText: string;
    placeholderText: string;
}

function TextFormElement(props: GivenProps) {
    const [backColorYes, setBackColorApperance] = useState(false);
    
    return (
        <div className={`${(backColorYes) ? "bg-[#3f3f3f]" : ""} p-[7.5px]`}>
            <label htmlFor={props.givenID}>{props.labelText}</label>
            <input type="text" 
                   id={props.givenID} placeholder={props.placeholderText} 
                   onFocus={() => setBackColorApperance(true)}
                   onBlur={() => setBackColorApperance(false)}/>
        </div>
    );
}

export default TextFormElement;