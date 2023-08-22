import { useEffect, useState, RefObject } from "react";
import { Button } from "primereact/button";
import { Dialog } from "primereact/dialog";
import { InputNumber, InputNumberValueChangeEvent } from "primereact/inputnumber";
import { InputText } from "primereact/inputtext";
import { Toast } from "primereact/toast";

interface LetterInputProps {
    /**
     * Toast reference for displaying alerts
     */
    toast: RefObject<Toast>,
    /**
     * Function to start solving the game
     * @param letters Map of every letter to the number present in the hand
     */
    startRunning: (letters: Map<string, number>) => void,
    /**
     * Whether the game is being solved or not
     */
    running: boolean
}

/**
 * Displays the letter number inputs, along with the input dialog and buttons
 * 
 * @component
 */
export default function LetterInput(props: LetterInputProps) {
    const UPPERCASE_LETTERS = [..."ABCDEFGHIJKLMNOPQRSTUVWXYZ"];
    const m = new Map();
    const num_letters = new Map<string, number>();
    const invalid = new Map<string, boolean>();
    const how_many = [13, 3, 3, 6, 18, 3, 4, 3, 12, 2, 2, 5, 3, 8, 11, 3, 2, 9, 6, 9, 6, 3, 3, 2, 3, 2];
    UPPERCASE_LETTERS.forEach((c, i) => {
        m.set(c, 0);
        num_letters.set(c, how_many[i]);
        invalid.set(c, false);
    });
    const [letterNums, setLetterNums] = useState<Map<string, number|null|undefined>>(m);
    const [lettersInvalid, setLettersInvalid] = useState<Map<string, boolean>>(invalid);
    const [typeInVisible, setTypeInVisible] = useState(false);
    const [typedIn, setTypedIn] = useState("");

    // Focus the input (requires a timeout since the dialog auto-focuses the "X")
    useEffect(() => {
        if (typeInVisible) {
            setTimeout(() => document.getElementById("typeIn")?.focus(), 100);
        }
    }, [typeInVisible]);

    /**
     * Callback when a number is changed for a specified letter
     * @param c The letter of the number being changed
     * @param e The input change event
     */
    const changeLetterNum = (c: string, e: InputNumberValueChangeEvent) => {
        const new_map = new Map(letterNums);
        new_map.set(c, e.value);
        setLetterNums(new_map);
        const n = Number(e.value);
        if (!isNaN(n) && num_letters.get(c)! < n) {
            const new_map_invalid = new Map(lettersInvalid);
            new_map_invalid.set(c, true);
            setLettersInvalid(new_map_invalid);
        }
        else {
            const new_map_invalid = new Map(lettersInvalid);
            new_map_invalid.set(c, false);
            setLettersInvalid(new_map_invalid);
        }
    }

    /**
     * Counts the occurences of the `letter` in the string `s`
     * @param letter Letter to count in `s`
     * @param s String to count the occurences of `letter` in
     * @returns The number of times `letter` appears in `s`
     */
    const count_letter_in_string = (letter: string, s: string) => {
        let count = 0;
        for (let i=0; i < s.length; i++) {
            if (s[i] === letter) {
                count += 1;
            }
        }
        return count;
    }

    /**
     * Callback when the "Use letters" button is clicked
     */
    const useLetters = () => {
        const new_map = new Map<string, number>();
        UPPERCASE_LETTERS.forEach(c => {
            new_map.set(c, count_letter_in_string(c, typedIn));
        });
        setLetterNums(new_map);
        setTypeInVisible(false);
        setTypedIn("");
    }

    /**
     * Callback to start solving the puzzle
     */
    const solve = () => {
        let s = 0;
        for (const value of letterNums.values()) {
            s += value ?? 0;
        }
        if (s < 2) {
            props.toast.current?.show({"severity": "warn", "summary": "Not enought letters", "detail": "More than one letter must be present."})
        }
        else {
            const letters = new Map<string, number>();
            UPPERCASE_LETTERS.forEach(c => {
                letters.set(c, letterNums.get(c) ?? 0);
            });
            props.startRunning(letters);
        }
    }
    
    return (
        <>
        <Dialog header="Type in letters" visible={typeInVisible} onHide={() => setTypeInVisible(false)}>
            <InputText value={typedIn} onChange={e => setTypedIn(e.target.value.toUpperCase())} keyfilter="alpha" id="typeIn"/>
            <br/>
            <Button label="Use letters" style={{marginTop: "5px", marginRight: "5px"}} onClick={useLetters}/>
            <Button label="Cancel" severity="secondary" onClick={() => {setTypedIn(""); setTypeInVisible(false)}}/>
        </Dialog>
        {UPPERCASE_LETTERS.map(c => {
            return <span style={{marginLeft: "5px", display: "inline-block"}} key={"span-"+c}><label htmlFor={"char-"+c} style={{display: "inline-block", minWidth: "20px"}}>{c}:</label>
            <InputNumber inputId={"char-"+c} value={letterNums.get(c)} onValueChange={e => changeLetterNum(c, e)} min={0} size={1} showButtons inputStyle={{padding: "5px", width: "3rem"}} incrementButtonClassName="input-button-type" decrementButtonClassName="input-button-type" className={lettersInvalid.get(c) ? "p-invalid" : undefined} style={{marginTop: "5px", paddingLeft: "5px"}}/></span>
        })}
        <br/>
        <Button label="Type in letters" style={{padding: "8px", marginTop: "5px", marginLeft: "15px", marginRight: "15px"}} onClick={() => setTypeInVisible(true)}/>
        <Button label="Solve" icon="pi pi-arrow-right" iconPos="right" style={{padding: "8px"}} severity="success" onClick={solve} loading={props.running}/>
        </>
    )
}