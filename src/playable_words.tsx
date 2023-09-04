import { Dialog } from "primereact/dialog";
import { TabView, TabPanel } from "primereact/tabview";
import PlayableWordsList from "./playable_words_list";

interface PlayableWordsProps {
    /**
     * The playable words
     */
    playableWords: null | {
        /**
         * Commonly accepted words
         */
        short: string[],
        /**
         * Words in the Scrabble dictionary
         */
        long: string[]
    },
    /**
     * Whether the popup should be visible
     */
    visible: boolean,
    /**
     * Sets the visibility of the popup
     * @param visible Whether the popup should be visible
     */
    setVisible: (visible: boolean) => void
}

/**
 * Shows different lists of playable words
 * 
 * @component
 */
export default function PlayableWords(props: PlayableWordsProps) {
    return (
        <>
        <Dialog header="Playable words" visible={props.visible} onHide={() => props.setVisible(false)} contentStyle={{width: "35vw", maxHeight: "85vh", overflowY: "hidden"}} pt={{mask: {onContextMenu: e => e.preventDefault()}}}>
            <TabView>
                <TabPanel header="Common words">
                    <PlayableWordsList playableWords={props.playableWords?.short} which="common"/>
                </TabPanel>
                <TabPanel header="Full Scrabble dictionary">
                    <PlayableWordsList playableWords={props.playableWords?.long} which="full"/>
                </TabPanel>
            </TabView>
        </Dialog>
        </>
    )
}