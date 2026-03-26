       IDENTIFICATION DIVISION.
       PROGRAM-ID. MULTISCREEN.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  APP-STATE.
           05  USER-NAME      PIC X(40) VALUE "".
           05  STATUS-MSG     PIC X(60) VALUE "Welcome!".

       SCREEN SECTION.
       01  MAIN-SCREEN.
           05  HEADER.
               10  TITLE      PIC X(30) VALUE "Home".
           05  CONTENT.
               10  MSG-TEXT   PIC X(60) USING STATUS-MSG.
           05  CONTROLS.
               10  SETTINGS-BTN VALUE "Settings" GO-TO-SCREEN SETTINGS-SCREEN.

       01  SETTINGS-SCREEN.
           05  HEADER.
               10  TITLE      PIC X(30) VALUE "Settings".
           05  CONTENT.
               10  NAME-FIELD PIC X(40) USING USER-NAME.
           05  CONTROLS.
               10  BACK-BTN   VALUE "Back" GO-TO-SCREEN MAIN-SCREEN.

       PROCEDURE DIVISION.
       MAIN-LOOP.
           STOP RUN.
