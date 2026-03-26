       IDENTIFICATION DIVISION.
       PROGRAM-ID. HELLO.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  APP-STATE.
           05  USER-NAME      PIC X(40) VALUE "".
           05  STATUS-MSG     PIC X(60) VALUE "Welcome to COBALT!".

       SCREEN SECTION.
       01  MAIN-SCREEN.
           05  HEADER.
               10  TITLE      PIC X(30) VALUE "Hello, COBALT!".
           05  CONTENT.
               10  NAME-FIELD PIC X(40) USING USER-NAME.
               10  GREET-BTN  VALUE "Greet" ON-ACTION PERFORM HANDLE-GREET.
           05  FOOTER.
               10  MSG-TEXT   PIC X(60) USING STATUS-MSG.

       PROCEDURE DIVISION.
       MAIN-LOOP.
           STOP RUN.

       HANDLE-GREET.
           DISPLAY "Hello!".
