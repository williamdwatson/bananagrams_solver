import { Accordion, AccordionTab } from "primereact/accordion";
import { Column } from "primereact/column";
import { DataTable } from "primereact/datatable";
import { Dialog } from "primereact/dialog";

interface StatisticProps {
    /**
     * Name of the statistic
     */
    name: string,
    /**
     * Value of the statistic
     */
    value: string|number
}

/**
 * Displays a single statistic
 * 
 * @component
 */
function Statistic(props: StatisticProps) {
    return (
        <div style={{marginBottom: "5px"}}>
            <span style={{fontWeight: "bold"}}>{props.name}: </span> {props.value}
        </div>
    )
}

interface StatisticTableProps {
    /**
     * The header of the type of data being displayed
     */
    what: string,
    /**
     * The values to display
     */
    values: Map<string|number, number>,
    /**
     * Datatype to pass to the table
     */
    datatype: "text"|"number"
}

/**
 * Displays a table of statistics
 * 
 * @component 
 */
function StatisticTable(props: StatisticTableProps) {
    return (
        <DataTable sortField="what" sortOrder={1} value={Array.from(props.values.entries()).map(([label, value]) => { return {what: label, number: value} })}>
            <Column field="what" header={props.what} sortable dataType={props.datatype}/>
            <Column field="number" header="Number" sortable dataType="number"/>
        </DataTable>
    )
}

interface PlayableWordsStatsProps {
    /**
     * List of playable words for which to calculate stats
     */
    playableWords: string[]|undefined,
    /**
     * Whether the popup is visible
     */
    visible: boolean,
    /**
     * Sets whether the popup is visible
     * @param visible Whether the popup should be visible
     */
    setVisible: (visible: boolean) => void
}

/**
 * Shows statistics about a set of playable words
 * 
 * @component
 */
export default function PlayableWordsStats(props: PlayableWordsStatsProps) {
    /**
     * Gets the most common element of `arr`
     * @param arr Array of elements from which to find the most common
     * @returns The most common element of `arr`; if there's a tie, the first will be returned
     */
    const mode = <T,>(arr: T[]) => {
        const num = new Map<T, number>();
        let maxNum = 1;
        let maxVal = arr[0];
        arr.forEach(val => {
            if (num.has(val)) {
                num.set(val, num.get(val)!+1);
                if (num.get(val)! > maxNum) {
                    maxNum = num.get(val)!;
                    maxVal = val;
                }
            }
            else {
                num.set(val, 1);
            }
        });
        return maxVal;
    }

    const lengths = new Map<number, number>();
    props.playableWords?.forEach(word => {
        const word_length = word.length;
        if (lengths.has(word_length)) {
            lengths.set(word_length, lengths.get(word_length)!+1);
        }
        else {
            lengths.set(word_length, 1);
        }
    });

    const starting_letters = new Map<string, number>();
    props.playableWords?.forEach(word => {
        const starting_letter = word.charAt(0);
        if (starting_letters.has(starting_letter)) {
            starting_letters.set(starting_letter, starting_letters.get(starting_letter)!+1);
        }
        else {
            starting_letters.set(starting_letter, 1);
        }
    });

    const ending_letters = new Map<string, number>();
    props.playableWords?.forEach(word => {
        const ending_letter = word.charAt(word.length-1);
        if (ending_letters.has(ending_letter)) {
            ending_letters.set(ending_letter, ending_letters.get(ending_letter)!+1);
        }
        else {
            ending_letters.set(ending_letter, 1);
        }
    });

    const used_letters = new Map<string, number>();
    props.playableWords?.forEach(word => {
        [...word].forEach(letter => {
            if (used_letters.has(letter)) {
                used_letters.set(letter, used_letters.get(letter)!+1);
            }
            else {
                used_letters.set(letter, 1);
            }
        });
    });

    return (
        <>
        {props.playableWords != null ? 
        <Dialog header="Statistics for playable words" visible={props.visible} onHide={() => props.setVisible(false)}>
            <Statistic name="Total number" value={props.playableWords.length}/>
            <Statistic name="Maximum length" value={Math.max(...props.playableWords.map(word => word.length))}/>
            <Statistic name="Average length" value={(props.playableWords.reduce((previousValue, currentWord) => previousValue + currentWord.length, 0)/props.playableWords.length).toFixed(2)}/>
            <Statistic name="Most common length" value={mode(props.playableWords.map(word => word.length))}/>
            <Statistic name="Most common starting letter" value={mode(props.playableWords.map(word => word.charAt(0)))}/>
            <Statistic name="Most common ending letter" value={mode(props.playableWords.map(word => word.charAt(word.length-1)))}/>
            <Statistic name="Most commonly occurring letter" value={mode(props.playableWords.flatMap(word => [...word]))}/>
            <Accordion>
                <AccordionTab header="Lengths">
                    <StatisticTable what="Length" values={lengths} datatype="number"/>
                </AccordionTab>
                <AccordionTab header="Starting letters">
                    <StatisticTable what="Letter" values={starting_letters} datatype="text"/>
                </AccordionTab>
                <AccordionTab header="Ending letters">
                    <StatisticTable what="Letter" values={ending_letters} datatype="text"/>
                </AccordionTab>
                <AccordionTab header="Used letters">
                    <StatisticTable what="Letter" values={used_letters} datatype="text"/>
                </AccordionTab>
            </Accordion>
        </Dialog>
        : null
        }
        </>
    )
}