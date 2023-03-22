import { Heading, HStack, Image, Link } from "@chakra-ui/react";

/**
 * Wiggle animation is defined in the style tag in `../pages/_document.tsx`
 * See: https://stackoverflow.com/questions/38132700/css-wiggle-shake-effect
 */
export default function Navbar() {
  return (
    <HStack>
      <Link
        isExternal={true}
        href="https://www.youtube.com/embed/dQw4w9WgXcQ?autoplay=1"
        animation="wiggle 2.5s infinite"
      >
        <Image
          src="badges.gif"
          alt="logo"
          w="50px"
          h="60px"
          borderRadius="0"
          transition="0.1s all"
          _hover={{
            transform: "scale(1.1)",
          }}
        />
      </Link>
      <Heading fontSize="3xl" fontFamily="Silkscreen" pt="2">
        Badges
      </Heading>
    </HStack>
  );
}
