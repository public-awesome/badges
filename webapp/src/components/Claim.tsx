import {
  useDisclosure,
  Box,
  Button,
  FormControl,
  FormErrorMessage,
  FormHelperText,
  FormLabel,
  HStack,
  Image,
  Input,
  Modal,
  ModalCloseButton,
  ModalContent,
  ModalOverlay,
  Text,
  VStack,
} from "@chakra-ui/react";
import { BadgeResponse } from "@steak-enjoyers/badges.js/types/codegen/Hub.types";
import { bech32 } from "bech32";
import { useEffect, useState } from "react";
import { QrReader } from "react-qr-reader";
import * as secp256k1 from "secp256k1";

import ScanIcon from "./ScanIcon";
import TxModal from "./TxModal";
import { getTimestampInSeconds, formatTimestamp, sha256, hexToBytes, bytesToHex } from "../helpers";
import { useStore } from "../store";

// https://stackoverflow.com/questions/5214127/css-technique-for-a-horizontal-line-with-words-in-the-middle
const hrStyle = {
  bg: "rgb(226, 232, 240)",
  content: `""`,
  display: "inline-block",
  height: "1px",
  position: "relative",
  verticalAlign: "middle",
  width: "calc(50% - 0.5rem - 9px)",
};

const fillerImageUrl = "https://via.placeholder.com/500?text=Image+Not+Available";
const fillerText = "Undefined";

enum Page {
  Credential = 1,
  Preview,
  Submit,
}

export default function Claim() {
  const store = useStore();

  // which page to display
  const [page, setPage] = useState(Page.Credential);

  // inputs - badge id
  const [idStr, setIdStr] = useState("");
  const [idValid, setIdValid] = useState<boolean | null>(null);
  const [idInvalidReason, setIdInvalidReason] = useState("");

  // inputs - key
  const [privkeyStr, setPrivkeyStr] = useState("");
  const [privkeyValid, setPrivkeyValid] = useState<boolean | null>(null);
  const [privkeyInvalidReason, setPrivkeyInvalidReason] = useState("");

  // inputs - owner
  const [owner, setOwner] = useState("");
  const [ownerValid, setOwnerValid] = useState<boolean | null>(null);
  const [ownerInvalidReason, setOwnerInvalidReason] = useState("");

  // whether webcam modal is open on the credentials page
  const { isOpen: isCameraOpen, onOpen: onCameraOpen, onClose: onCameraClose } = useDisclosure();

  // whether tx modal is open on the submit page
  const { isOpen: isTxModalOpen, onOpen: onTxModalOpen, onClose: onTxModalClose } = useDisclosure();

  // values on the preview page
  const [badge, setBadge] = useState<BadgeResponse>();

  // whenever input id is changed, validate it
  useEffect(() => {
    function setIdValidNull() {
      setIdValid(null);
      setIdInvalidReason("");
      console.log("empty id");
    }

    function setIdValidTrue() {
      setIdValid(true);
      setIdInvalidReason("");
      console.log(`id "${idStr}" is valid`);
    }

    function setIdValidFalse(reason: string) {
      setIdValid(false);
      setIdInvalidReason(reason);
      console.log(`invalid id "${idStr}": ${reason}`);
    }

    //--------------------
    // stateless checks
    //--------------------

    if (idStr === "") {
      return setIdValidNull();
    }

    const id = Number(idStr);

    if (!Number.isInteger(id)) {
      return setIdValidFalse("id must be an integer!");
    }

    if (id < 1) {
      return setIdValidFalse("id cannot be zero!");
    }

    if (!!store.badgeCount && id > store.badgeCount) {
      return setIdValidFalse(
        `id cannot be greater than the current badge count! (count: ${store.badgeCount})`
      );
    }

    //--------------------
    // stateful checks
    //--------------------

    // skip if the query client isn't initialized
    if (!store.wasmClient) {
      return setIdValidNull();
    }

    store.getBadge(id).then((badge) => {
      if (badge.rule !== "by_keys" && !("by_key" in badge.rule)) {
        return setIdValidFalse("id is valid but this badge is not publicly mintable!");
      }

      if (badge.expiry && getTimestampInSeconds() > badge.expiry) {
        return setIdValidFalse(
          `id is valid but minting deadline has already elapsed! (deadline: ${formatTimestamp(
            badge.expiry
          )})`
        );
      }

      if (badge.max_supply && badge.current_supply >= badge.max_supply) {
        return setIdValidFalse(
          `id is valid but max supply has already been reached! (max supply: ${badge.max_supply})`
        );
      }

      return setIdValidTrue();
    });
  }, [idStr, store.wasmClient]);

  // whenever input key is changed, we need to validate it
  useEffect(() => {
    function setPrivkeyValidNull() {
      setPrivkeyValid(null);
      setPrivkeyInvalidReason("");
      console.log("empty key");
    }

    function setPrivkeyValidTrue() {
      setPrivkeyValid(true);
      setPrivkeyInvalidReason("");
      console.log(`key "${privkeyStr}" is valid`);
    }

    function setPrivkeyValidFalse(reason: string) {
      setPrivkeyValid(false);
      setPrivkeyInvalidReason(reason);
      console.log(`invalid key "${privkeyStr}": ${reason}`);
    }

    //--------------------
    // stateless checks
    //--------------------

    if (privkeyStr === "") {
      return setPrivkeyValidNull();
    }

    const bytes = Buffer.from(privkeyStr, "hex");

    // A string is a valid hex-encoded bytearray if it can be decoded to a Buffer, and the string
    // has exactly twice as many bytes as the number of the Buffer's bytes.
    if (bytes.length * 2 != privkeyStr.length) {
      return setPrivkeyValidFalse("not a valid hex string!");
    }

    try {
      if (!secp256k1.privateKeyVerify(bytes)) {
        return setPrivkeyValidFalse("not a valid secp256k1 private key!");
      }
    } catch (err) {
      return setPrivkeyValidFalse(`not a valid secp256k1 private key: ${err}`);
    }

    //--------------------
    // stateful checks
    //--------------------

    // skip if the query client isn't initialized
    if (!store.wasmClient) {
      return setPrivkeyValidNull();
    }

    // Now we know the key is a valid secp256k1 privkey, we need to check whether it is eligible for
    // claiming the badge.
    // Firstly, if we don't already have a valid badge id, it's impossible to determine to badge's
    // eligibility. Simply return null in this case.
    if (!!!idValid) {
      return setPrivkeyValidNull();
    }

    const pubkeyStr = bytesToHex(secp256k1.publicKeyCreate(hexToBytes(privkeyStr)));

    // this block of code is fucking atrocious, but "it just works"
    store.getBadge(Number(idStr)).then((badge) => {
      if (badge.rule === "by_keys") {
        store
          .isKeyWhitelisted(Number(idStr), pubkeyStr)
          .then((isWhitelisted) => {
            if (isWhitelisted) {
              return setPrivkeyValidTrue();
            } else {
              return setPrivkeyValidFalse(`this key is not eligible to claim badge #${idStr}`);
            }
          })
          .catch((err) => {
            return setPrivkeyValidFalse(
              `failed to check this key's eligibility to claim badge #${idStr}: ${err}`
            );
          });
      } else if ("by_key" in badge.rule) {
        if (pubkeyStr === badge.rule["by_key"]) {
          return setPrivkeyValidTrue();
        } else {
          return setPrivkeyValidFalse(`this key is not eligible to claim badge #${idStr}`);
        }
      } else {
        return setPrivkeyValidFalse(`this key is not eligible to claim badge #${idStr}`);
      }
    });
  }, [privkeyStr, idStr, idValid, store.wasmClient]);

  // whenver input owner address is changed, we need to validate it
  useEffect(() => {
    function setOwnerValidNull() {
      setOwnerValid(null);
      setOwnerInvalidReason("");
      console.log("empty key");
    }

    function setOwnerValidTrue() {
      setOwnerValid(true);
      setOwnerInvalidReason("");
      console.log(`key "${privkeyStr}" is valid`);
    }

    function setOwnerValidFalse(reason: string) {
      setOwnerValid(false);
      setOwnerInvalidReason(reason);
      console.log(`invalid key "${privkeyStr}": ${reason}`);
    }

    //--------------------
    // stateless checks
    //--------------------

    if (owner === "") {
      return setOwnerValidNull();
    }

    try {
      const { prefix } = bech32.decode(owner);
      if (prefix !== store.networkConfig!.prefix) {
        return setOwnerValidFalse(
          `address has incorrect prefix: expecting ${store.networkConfig!.prefix}, found ${prefix}`
        );
      }
    } catch (err) {
      return setOwnerValidFalse(`not a valid bech32 address: ${err}`);
    }

    //--------------------
    // stateful checks
    //--------------------

    // skip if the query client isn't initialized
    if (!store.wasmClient) {
      return setOwnerValidNull();
    }

    // Now we know the owner is a valid bech32 address, we need to check whether it is eligible for
    // claiming the badge.
    // Firstly, if we don't already have a valid badge id, it's impossible to determine to badge's
    // eligibility. Simply return null in this case.
    if (!!!idValid) {
      return setOwnerValidNull();
    }

    store
      .isOwnerEligible(Number(idStr), owner)
      .then((eligible) => {
        if (eligible) {
          return setOwnerValidTrue();
        } else {
          return setOwnerValidFalse(`this address is not eligible to claim badge #${idStr}`);
        }
      })
      .catch((err) => {
        return setOwnerValidFalse(
          `failed to check this address' eligibility to claim badge #${idStr}: ${err}`
        );
      });
  }, [owner, idStr, idValid, store.wasmClient]);

  // if the id has been updated, we need to update the metadata displayed on the preview page
  // only update if the id is valid AND wasm client has been initialized
  useEffect(() => {
    if (!store.wasmClient) {
      console.log(`wasm client is uninitialized, setting badge to undefined`);
      return setBadge(undefined);
    }
    if (!idValid) {
      console.log(`invalid badge id "${idStr}", setting badge to undefined`);
      return setBadge(undefined);
    }
    store
      .getBadge(Number(idStr))
      .then((badge) => {
        console.log(`successfully fetched badge with id "${idStr}"! badge:`, badge);
        setBadge(badge);
      })
      .catch((err) => {
        console.log(`failed to fetch badge with id "${idStr}"! reason:`, err);
        setBadge(undefined);
      });
  }, [idStr, idValid, store.wasmClient]);

  // when the component is first mounted, we check the URL query params and auto-fill id and key
  useEffect(() => {
    const url = window.location.href;
    const split = url.split("?");
    const params = new URLSearchParams(split[1]);
    setIdStr(params.get("id") ?? "");
    setPrivkeyStr(params.get("key") ?? "");
  }, []);

  // if image url starts with `ipfs://...`, we grab the CID and return it with Larry's pinata gateway
  // otherwise, we return the url unmodified
  function parseImageUrl(url: string) {
    const ipfsPrefix = "ipfs://";
    if (url.startsWith(ipfsPrefix)) {
      const cid = url.slice(ipfsPrefix.length);
      return `https://ipfs-gw.stargaze-apis.com/ipfs/${cid}`;
    } else {
      return url;
    }
  }

  async function getMintMsg() {
    const privKey = Buffer.from(privkeyStr, "hex");
    const msg = `claim badge ${idStr} for user ${owner}`;
    const msgBytes = Buffer.from(msg, "utf8");
    const msgHashBytes = sha256(msgBytes);
    const { signature } = secp256k1.ecdsaSign(msgHashBytes, privKey);

    const badge = await store.getBadge(Number(idStr));

    if (badge.rule === "by_keys") {
      return {
        mint_by_keys: {
          id: Number(idStr),
          owner,
          pubkey: Buffer.from(secp256k1.publicKeyCreate(privKey)).toString("hex"),
          signature: Buffer.from(signature).toString("hex"),
        },
      };
    } else if ("by_key" in badge.rule) {
      return {
        mint_by_key: {
          id: Number(idStr),
          owner,
          signature: Buffer.from(signature).toString("hex"),
        },
      };
    } else {
      return {
        mint_by_minter: {
          id: Number(idStr),
          owners: [owner],
        },
      };
    }
  }

  // when user closes the tx modal, we reset the page: revert back to the credentials page, and
  // empty the inputs
  function onClosingTxModal() {
    setPage(Page.Credential);
    setIdStr("");
    setIdValid(null);
    setPrivkeyStr("");
    setPrivkeyValid(null);
    setOwner("");
    setOwnerValid(null);
    onTxModalClose();
  }

  const credentialPage = (
    <Box>
      <Text mb="4">1️⃣ Enter your claim credentials</Text>
      <Button w="100%" minH="8rem" onClick={onCameraOpen}>
        <HStack>
          <ScanIcon w="2.5rem" h="2.5rem" mr="2" />
          <Text>Scan QR code</Text>
        </HStack>
      </Button>
      <Modal isOpen={isCameraOpen} onClose={onCameraClose}>
        <ModalOverlay />
        <ModalContent bg="rgba(0,0,0,0)" boxShadow="" maxW="min(90%, 600px)" border="none">
          <QrReader
            constraints={{
              facingMode: "environment",
            }}
            onResult={(result, _error) => {
              if (!!result) {
                const text = result.getText();

                const split = text.split("?");
                if (split.length !== 2) {
                  return alert(`!! invalid QR !!\nnot a valid URL with a query string: ${text}`);
                }

                const params = new URLSearchParams(split[1]);
                if (!(params.has("id") && params.has("key"))) {
                  return alert(
                    `!! invalid QR !!\nquery string does not contain both parameters "id" and "key"`
                  );
                }

                setIdStr(params.get("id")!);
                setPrivkeyStr(params.get("key")!);
                onCameraClose();
              }
            }}
            videoContainerStyle={{
              padding: "0",
              borderRadius: "var(--chakra-radii-lg)",
              // TODO: The video takes a 1-2 seconds to load. At the mean time the box shadow is
              // very ugly. Is there any way to delay the displaying of box shadow after the video
              // is loaded?
              boxShadow: "var(--chakra-shadows-dark-lg)",
            }}
            videoStyle={{
              position: "relative",
              borderRadius: "var(--chakra-radii-lg)",
            }}
          />
          <ModalCloseButton
            size="lg"
            top="var(--chakra-space-3)"
            fill="white"
            bg="whiteAlpha.700"
            _hover={{ bg: "whiteAlpha.800" }}
            _active={{ bg: "whiteAlpha.900" }}
          />
        </ModalContent>
      </Modal>
      <Text
        my="4"
        _before={{
          right: "0.5rem",
          ml: "2",
          ...hrStyle,
        }}
        _after={{
          left: "0.5rem",
          mr: "2",
          ...hrStyle,
        }}
      >
        or
      </Text>
      <Text mb="4">Enter manually:</Text>
      <FormControl mb="4" isInvalid={idValid !== null && !idValid}>
        <FormLabel>id</FormLabel>
        <Input
          placeholder="a number"
          value={idStr}
          onChange={(event) => {
            setIdStr(event.target.value);
          }}
        />
        {idValid !== null ? (
          idValid ? (
            <FormHelperText>✅ valid id</FormHelperText>
          ) : (
            <FormErrorMessage>{idInvalidReason}</FormErrorMessage>
          )
        ) : null}
      </FormControl>
      <FormControl mb="4" isInvalid={privkeyValid !== null && !privkeyValid}>
        <FormLabel>key</FormLabel>
        <Input
          placeholder="hex-encoded string"
          value={privkeyStr}
          onChange={(event) => {
            setPrivkeyStr(event.target.value);
          }}
        />
        {privkeyValid !== null ? (
          privkeyValid ? (
            <FormHelperText>✅ valid key</FormHelperText>
          ) : (
            <FormErrorMessage>{privkeyInvalidReason}</FormErrorMessage>
          )
        ) : null}
      </FormControl>
      <Button
        variant="outline"
        onClick={() => setPage(Page.Preview)}
        isDisabled={!(!!idValid && !!privkeyValid)}
      >
        Next
      </Button>
    </Box>
  );

  const previewPage = (
    <Box>
      <Text mb="4">2️⃣ Preview of your badge</Text>
      <Image
        src={parseImageUrl(badge?.metadata.image ?? fillerImageUrl)}
        alt="badge-image"
        w="100%"
        mx="auto"
        mb="4"
        border="1px solid rgb(226, 232, 240)"
        borderRadius="xl"
      />
      <VStack alignItems="start" mb="4">
        <Box>
          <Text fontSize="sm" fontWeight="400">
            name
          </Text>
          <Text>{badge?.metadata.name ?? fillerText}</Text>
        </Box>
        <Box>
          <Text fontSize="sm" fontWeight="400">
            description
          </Text>
          <Text lineHeight="1.25" py="1">
            {badge?.metadata.description ?? fillerText}
          </Text>
        </Box>
        {/* <Box>
          <Text fontSize="sm" fontWeight="400">
            creator
          </Text>
          <Text>{badge?.manager ?? fillerText}</Text>
        </Box> */}
        <Box>
          <Text fontSize="sm" fontWeight="400">
            current supply
          </Text>
          <Text>{badge?.current_supply ?? fillerText}</Text>
        </Box>
        <Box>
          <Text fontSize="sm" fontWeight="400">
            max supply
          </Text>
          <Text>{badge ? badge.max_supply ?? "No max supply" : fillerText}</Text>
        </Box>
        <Box>
          <Text fontSize="sm" fontWeight="400">
            minting deadline
          </Text>
          <Text>
            {badge ? (badge.expiry ? formatTimestamp(badge.expiry) : "No deadline") : fillerText}
          </Text>
        </Box>
      </VStack>
      <Button variant="outline" mr="1" onClick={() => setPage(Page.Credential)}>
        Back
      </Button>
      <Button variant="outline" ml="1" onClick={() => setPage(Page.Submit)}>
        Next
      </Button>
    </Box>
  );

  const submitPage = (
    <Box>
      <Text mb="4">3️⃣ Claim now!</Text>
      <FormControl mb="4" isInvalid={ownerValid !== null && !ownerValid}>
        <FormLabel>Your Stargaze address</FormLabel>
        <Input
          placeholder="stars1..."
          onChange={(event) => {
            setOwner(event.target.value);
          }}
        />
        {ownerValid !== null ? (
          ownerValid ? (
            <FormHelperText>✅ valid address</FormHelperText>
          ) : (
            <FormErrorMessage>{ownerInvalidReason}</FormErrorMessage>
          )
        ) : (
          <FormHelperText>
            Unfortunately, autofill by connecting to a wallet app isn&apos;t supported yet. Please
            copy-paste your address here.
          </FormHelperText>
        )}
      </FormControl>
      <Button variant="outline" mr="1" onClick={() => setPage(Page.Preview)}>
        Back
      </Button>
      <Button
        variant="outline"
        ml="1"
        onClick={onTxModalOpen}
        isLoading={false}
        isDisabled={false} // TODO
      >
        Submit
      </Button>
      <TxModal isOpen={isTxModalOpen} onClose={onClosingTxModal} getMsg={getMintMsg} />
    </Box>
  );

  const pages = {
    [Page.Credential]: credentialPage,
    [Page.Preview]: previewPage,
    [Page.Submit]: submitPage,
  };

  return <Box px="2">{pages[page]}</Box>;
}
