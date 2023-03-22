import Head from "next/head";
import { NextPage } from "next";

import Claim from "../components/Claim";

const IndexPage: NextPage = () => {
  return (
    <>
      <Head>
        <title>badges | claim</title>
      </Head>
      <Claim />
    </>
  );
};

export default IndexPage;
