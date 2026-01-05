import { createContext, useContext } from "react";
export const AppContext = createContext({
    windowId: 0
});
export const useAppContext = () => {
    return useContext(AppContext);
};
