       IDENTIFICATION DIVISION.
       PROGRAM-ID. CLOCK.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  APP-STATE.
           05  CURRENT-DATE    PIC X(10) VALUE "00000000".
           05  CURRENT-TIME    PIC X(8)  VALUE "000000".
           05  DAY-NUM         PIC X(1)  VALUE "0".
           05  DAY-NAME        PIC X(10) VALUE "Unknown".
           05  STATUS-MSG      PIC X(60) VALUE "Press Refresh".

       SCREEN SECTION.
       01  MAIN-SCREEN.
           05  HEADER.
               10  TITLE       PIC X(30) VALUE "Clock".
           05  DATE-AREA.
               10  DATE-LABEL  PIC X(10) VALUE "Date".
               10  DATE-DISP   PIC X(10) USING CURRENT-DATE.
           05  TIME-AREA.
               10  TIME-LABEL  PIC X(10) VALUE "Time".
               10  TIME-DISP   PIC X(8) USING CURRENT-TIME.
           05  DAY-AREA.
               10  DOW-LABEL   PIC X(10) VALUE "Day".
               10  DOW-DISP    PIC X(10) USING DAY-NAME.
           05  CONTROLS.
               10  REFRESH-BTN VALUE "Refresh" ON-ACTION PERFORM HANDLE-REFRESH.
           05  STATUS-BAR.
               10  MSG-TEXT    PIC X(60) USING STATUS-MSG.

       PROCEDURE DIVISION.
       MAIN-LOOP.
           STOP RUN.

       HANDLE-REFRESH.
           ACCEPT CURRENT-DATE FROM DATE.
           ACCEPT CURRENT-TIME FROM TIME.
           ACCEPT DAY-NUM FROM DAY-OF-WEEK.
           PERFORM SET-DAY-NAME.
           MOVE "Updated" TO STATUS-MSG.

       SET-DAY-NAME.
           EVALUATE DAY-NUM
               WHEN "1"
                   MOVE "Monday" TO DAY-NAME
               WHEN "2"
                   MOVE "Tuesday" TO DAY-NAME
               WHEN "3"
                   MOVE "Wednesday" TO DAY-NAME
               WHEN "4"
                   MOVE "Thursday" TO DAY-NAME
               WHEN "5"
                   MOVE "Friday" TO DAY-NAME
               WHEN "6"
                   MOVE "Saturday" TO DAY-NAME
               WHEN "7"
                   MOVE "Sunday" TO DAY-NAME
               WHEN OTHER
                   MOVE "Unknown" TO DAY-NAME
           END-EVALUATE.
