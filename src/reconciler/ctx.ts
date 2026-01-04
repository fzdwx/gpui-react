import {createContext, useContext} from "react";

interface AppContext {
    windowId: number
}

export const AppContext = createContext<AppContext>({
    windowId: 0
})

export const useAppContext = () => {
    return useContext(AppContext)
}