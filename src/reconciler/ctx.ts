import {createContext, useContext} from "react";

interface AppContext {
    windowsID:number
}

export const AppContext = createContext<AppContext>({
    windowsID:0
})

export const useAppContext = () => {
    return useContext(AppContext)
}