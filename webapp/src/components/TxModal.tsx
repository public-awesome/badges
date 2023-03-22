import { Box, Button, Flex, Link, Spinner, Text } from "@chakra-ui/react";
import { FC, useState, useEffect } from "react";

import ModalWrapper from "./ModalWrapper";
import SuccessIcon from "./TxSuccessIcon";
import FailedIcon from "./TxFailedIcon";
import ExternalLinkIcon from "./ExternalLinkIcon";
import { truncateString } from "../helpers";
import { useStore } from "../store";

function SpinnerWrapper() {
  return (
    <Spinner thickness="6px" speed="1s" emptyColor="transparent" color="brand.red" size="xl" />
  );
}

function TxHashText(txhash: string, url: string) {
  return (
    <Flex>
      <Text variant="dimmed" ml="auto" mr="3">
        Tx Hash
      </Text>
      <Link isExternal href={url} ml="3" mr="auto" my="auto" textUnderlineOffset="0.3rem">
        {truncateString(txhash, 6, 6)}
        <ExternalLinkIcon
          ml="2"
          style={{
            transform: "translateY(-2.4px)",
          }}
        />
      </Link>
    </Flex>
  );
}

function TxFailedText(error: any) {
  return (
    <Text mx="auto" px="12">
      {error}
    </Text>
  );
}

function CloseButton(showCloseBtn: boolean, onClick: () => void) {
  return showCloseBtn ? (
    <Button variant="outline" mt="12" onClick={onClick}>
      Close
    </Button>
  ) : null;
}

type Props = {
  getMsg: () => Promise<any>;
  isOpen: boolean;
  onClose: () => void;
};

const TxModal: FC<Props> = ({ getMsg, isOpen, onClose }) => {
  const store = useStore();
  const [showCloseBtn, setShowCloseBtn] = useState<boolean>(false);
  const [txStatusHeader, setTxStatusHeader] = useState<string>();
  const [txStatusIcon, setTxStatusIcon] = useState<JSX.Element>();
  const [txStatusDetail, setTxStatusDetail] = useState<JSX.Element>();

  useEffect(() => {
    setTxStatusHeader("Broadcasting Transaction...");
    setTxStatusIcon(SpinnerWrapper());
    setTxStatusDetail(<Text>Should be done in a few seconds 😉</Text>);
    setShowCloseBtn(false);
  }, [isOpen]);

  useEffect(() => {
    if (isOpen) {
      getMsg()
        .then((msg) => {
          console.log("created execute msg:", msg);
          store
            .wasmClient!.execute(store.senderAddr!, store.networkConfig!.hub, msg, "auto", "", [])
            .then((result) => {
              setTxStatusHeader("Transaction Successful");
              setTxStatusDetail(
                TxHashText(
                  result.transactionHash,
                  store.networkConfig!.getExplorerUrl(result.transactionHash)
                )
              );
              setTxStatusIcon(<SuccessIcon h="80px" w="80px" />);
              setShowCloseBtn(true);
            })
            .catch((error) => {
              setTxStatusHeader("Transaction Failed");
              setTxStatusIcon(<FailedIcon h="80px" w="80px" />);
              setTxStatusDetail(TxFailedText(error));
              setShowCloseBtn(true);
            });
        })
        .catch((error) => {
          setTxStatusHeader("Transaction Failed");
          setTxStatusIcon(<FailedIcon h="80px" w="80px" />);
          setTxStatusDetail(TxFailedText(error));
          setShowCloseBtn(true);
        });
    }
  }, [isOpen]);

  return (
    <ModalWrapper showHeader={false} isOpen={isOpen} onClose={onClose}>
      <Box w="100%" textAlign="center">
        <Text fontSize="xl" textStyle="minibutton" mt="10">
          {txStatusHeader}
        </Text>
        <Flex w="100%" h="150px" align="center" justify="center">
          {txStatusIcon}
        </Flex>
        <Box mt="3" mb="10">
          {txStatusDetail}
          {CloseButton(showCloseBtn, onClose)}
        </Box>
      </Box>
    </ModalWrapper>
  );
};

export default TxModal;
