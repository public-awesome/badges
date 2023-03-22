import { ChakraProvider } from "@chakra-ui/react";
import { AppProps } from "next/app";

import Layout from "../components/Layout";
import theme from "../theme";

const BadgesApp = ({ Component, pageProps }: AppProps) => {
  return (
    <ChakraProvider theme={theme}>
      <Layout>
        <Component {...pageProps} />
      </Layout>
    </ChakraProvider>
  );
};

export default BadgesApp;
