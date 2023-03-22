import Head from "next/head";
import { NextPage } from "next";

import Gallery from "../components/Gallery";

const IndexPage: NextPage = () => {
  return (
    <>
      <Head>
        <title>badges | gallery</title>
      </Head>
      <Gallery />
    </>
  );
};

export default IndexPage;
