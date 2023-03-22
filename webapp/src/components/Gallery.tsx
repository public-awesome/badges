import { useEffect } from "react";
import Masonry, { ResponsiveMasonry } from "react-responsive-masonry";
import { Box, Image } from "@chakra-ui/react";
import { convertFromIPFSUrl } from "src/helpers";
import { useStore } from "src/store";

export default function Gallery() {
  const store = useStore();

  useEffect(() => {
    if (store.wasmClient) {
      store.getAllBadges();
    }
  }, [store.wasmClient]);

  return (
    <Box mx={-2}>
      <ResponsiveMasonry columnsCountBreakPoints={{ 0: 1, 550: 2 }}>
        <Masonry>
          {Object.values(store.badges).map((badge) => (
            <Box key={badge.id} borderWidth="1px" borderRadius="lg" overflow="hidden" mx={2} mb={4}>
              <Image
                maxW="min(fill, 300)"
                maxH="fill"
                objectFit="contain"
                src={convertFromIPFSUrl(badge.metadata.image)}
                alt={badge.metadata.name || String(badge.id)}
              />
              <hr />
              <Box p="5">
                <Box display="flex" alignItems="baseline">
                  <Box fontWeight="semibold" as="h4" lineHeight="tight" noOfLines={1}>
                    {`#${badge.id}`} {badge.metadata.name ?? "Unnamed"}
                  </Box>
                </Box>

                <Box mt="1" fontSize="sm">
                  <Box as="span" color="gray.600" mr={1}>
                    current supply:
                  </Box>
                  {badge.current_supply}
                </Box>
              </Box>
            </Box>
          ))}
        </Masonry>
      </ResponsiveMasonry>
    </Box>
  );
}
