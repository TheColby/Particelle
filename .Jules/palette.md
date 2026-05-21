## YYYY-MM-DD - [Title]
 **Learning:** [insight]
 **Action:** [application]
## 2024-05-21 - CLI Rendering Progress Indicator
 **Learning:** When a completely headless audio processing tool takes some time to process a large file, the absence of any feedback can leave the user wondering if the app is hanging.
 **Action:** We can use simple `std::io::IsTerminal` to conditionally print lightweight text-based progress in the terminal during the render loop.
