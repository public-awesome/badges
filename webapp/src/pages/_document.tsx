import Document, { Html, Head, Main, NextScript } from "next/document";

class BadgesDocument extends Document {
  override render() {
    return (
      <Html>
        <Head>
          <link rel="preconnect" href="https://fonts.googleapis.com" />
          <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="" />
          <link
            rel="stylesheet"
            href="https://fonts.googleapis.com/css2?family=Nunito:wght@400;700&family=Silkscreen:wght@700&display=swap"
          />
          <style>
            {`
              @keyframes wiggle {
                0%   { transform: rotate(0deg);  }
                80%  { transform: rotate(0deg);  }
                85%  { transform: rotate(5deg);  }
                95%  { transform: rotate(-5deg); }
                100% { transform: rotate(0deg);  }
              }
            `}
          </style>
        </Head>
        <body style={{ overflowY: "scroll" }}>
          <Main />
          <NextScript />
        </body>
      </Html>
    );
  }
}

export default BadgesDocument;
