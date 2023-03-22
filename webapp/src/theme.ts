import { extendTheme } from "@chakra-ui/react";

const defaultSansSerif = "-apple-system,BlinkMacSystemFont,Segoe UI,Roboto,Helvetica Neue,Arial,Noto Sans,sans-serif";
const defaultEmoji = "Apple Color Emoji,Segoe UI Emoji,Segoe UI Symbol,Noto Color Emoji";

export default extendTheme({
  fonts: {
    heading: `Silkscreen,${defaultSansSerif},${defaultEmoji}`,
    body: `Nunito,${defaultSansSerif},${defaultEmoji}`,
    mono: "Menlo,monospace",
  },
  styles: {
    global: {
      p: {
        fontWeight: "700",
      },
    },
  },
});
