import { createTheme } from "@mui/material/styles";

export const appShellTheme = createTheme({
  palette: {
    mode: "light",
    primary: {
      main: "#29583f",
      dark: "#1f7f5c",
      contrastText: "#fffdf8",
    },
    secondary: {
      main: "#7a5727",
    },
    background: {
      default: "#f7f4ec",
      paper: "rgba(255, 252, 247, 0.9)",
    },
    text: {
      primary: "#1d1a16",
      secondary: "#433d37",
    },
  },
  shape: {
    borderRadius: 18,
  },
  typography: {
    fontFamily: '"IBM Plex Sans", "Segoe UI", sans-serif',
    button: {
      textTransform: "none",
      fontWeight: 700,
    },
  },
});
