import { Flex, Spacer } from "@chakra-ui/react";
import { FC, ReactNode, useEffect } from "react";

import { useStore } from "src/store";

import Navbar from "./Navbar";
import Footer from "./Footer";

// Starting React 18, we need to manually define the Props interface
// https://stackoverflow.com/questions/59106742/typescript-error-property-children-does-not-exist-on-type-reactnode
interface Props {
  children: ReactNode;
}

const Layout: FC<Props> = ({ children }) => {
  const store = useStore();

  // initialize the store when the app is mounted.
  // this includes picking a random account to sign the tx, and initialize the wasm client
  //
  // include an empty dependency array so that this only runs once
  useEffect(() => {
    store.init();
  }, []);

  return (
    <Flex minHeight="100vh" direction="column" maxW="600px" mx="auto">
      <Navbar />
      {children}
      <Spacer />
      <Footer />
    </Flex>
  );
};

export default Layout;
