import {
  useDisclosure,
  Box,
  Button,
  Drawer,
  DrawerOverlay,
  DrawerContent,
  Flex,
  HStack,
  Spacer,
  VStack,
} from "@chakra-ui/react";
import { useRef } from "react";

import BurgerIcon from "./BurgerIcon";
import CloseIcon from "./CloseIcon";
import NavbarLink from "./NavbarLink";
import NavbarLogo from "./NavbarLogo";

export default function Navbar() {
  const { isOpen, onOpen, onClose } = useDisclosure();

  // https://github.com/DefinitelyTyped/DefinitelyTyped/issues/35572#issuecomment-493942129
  const btnRef = useRef() as React.MutableRefObject<HTMLButtonElement>;

  return (
    <Box w="100%" mb="4">
      <Flex justify="space-between" align="center" pl="2" pr="0" py="4">
        <NavbarLogo />
        <Spacer />
        <Button
          variant="simple"
          minH="3rem"
          minW="3rem"
          p="2"
          _hover={{
            background: "#e1ebff",
          }}
          ref={btnRef}
          onClick={onOpen}
        >
          <BurgerIcon h="1.5rem" w="1.5rem" color="black" />
        </Button>
      </Flex>
      <Drawer
        isOpen={isOpen}
        onClose={onClose}
        finalFocusRef={btnRef}
        placement="left"
        size="sm"
        preserveScrollBarGap={true}
      >
        <DrawerOverlay />
        <DrawerContent>
          <HStack w="100%" px="12" py="4" align="stretch">
            <NavbarLogo />
            <Spacer></Spacer>
            <Button
              variant="simple"
              onClick={onClose}
              h="100%"
              p="2"
              transition="0.1s all"
              _hover={{
                background: "#e1ebff",
              }}
            >
              <CloseIcon width="2.125rem" height="2.125rem" />
            </Button>
          </HStack>
          <hr />
          <VStack height="100%" px="12" mt="8" spacing="8" align="left">
            <NavbarLink text="Claim" href="/" fontSize="xx-large" />
            <NavbarLink text="Create" href="/create" fontSize="xx-large" disabled={true} />
            <NavbarLink text="Gallery" href="/gallery" fontSize="xx-large" />
          </VStack>
        </DrawerContent>
      </Drawer>
      <hr />
    </Box>
  );
}
