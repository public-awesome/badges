import { chakra, Heading } from "@chakra-ui/react";
import NextLink from "next/link";
import { useRouter } from "next/router";
import { FC } from "react";

const DisabledAnchor: FC = ({ children }) => {
  return (
    <chakra.a transition="0.1s all" color="#adb5bd" pointerEvents="none">
      {children}
    </chakra.a>
  );
};

const ActiveAnchor: FC = ({ children }) => {
  return (
    <chakra.a
      transition="0.1s all"
      color="black"
      textDecoration="underline"
      textUnderlineOffset="6px"
      textDecorationThickness="3.5px"
      pointerEvents="none"
    >
      {children}
    </chakra.a>
  );
};

const EnabledAnchor: FC<{ href?: string }> = ({ children, href }) => {
  return (
    <chakra.a
      transition="0.1s all"
      color="#adb5bd"
      _hover={{
        color: "black",
      }}
      href={href}
    >
      {children}
    </chakra.a>
  );
};

type Props = {
  text: string;
  href: string;
  fontSize?: string;
  disabled?: boolean;
};

const NavbarLink: FC<Props> = ({ text, href, fontSize = "xl", disabled = false }) => {
  const { asPath } = useRouter();

  const content = (
    <Heading fontSize={fontSize}>
      {text}
      {disabled ? <sup> (soon)</sup> : null}
    </Heading>
  );

  return (
    <NextLink href={href} passHref>
      {disabled ? (
        <DisabledAnchor>{content}</DisabledAnchor>
      ) : asPath === href ? (
        <ActiveAnchor>{content}</ActiveAnchor>
      ) : (
        <EnabledAnchor>{content}</EnabledAnchor>
      )}
    </NextLink>
  );
};

export default NavbarLink;
