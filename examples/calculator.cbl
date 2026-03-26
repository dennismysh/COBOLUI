       IDENTIFICATION DIVISION.
       PROGRAM-ID. CALCULATOR.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  APP-STATE.
           05  OPERAND-A       PIC 9(4) VALUE 0.
           05  OPERAND-B       PIC 9(4) VALUE 0.
           05  RESULT-VAL      PIC 9(8) VALUE 0.
           05  STATUS-MSG      PIC X(60) VALUE "Ready".

       SCREEN SECTION.
       01  MAIN-SCREEN.
           05  HEADER.
               10  TITLE       PIC X(30) VALUE "Calculator".
           05  INPUT-AREA.
               10  A-FIELD     PIC 9(4) USING OPERAND-A.
               10  B-FIELD     PIC 9(4) USING OPERAND-B.
           05  RESULT-AREA.
               10  RES-DISPLAY PIC 9(8) USING RESULT-VAL.
           05  CONTROLS.
               10  ADD-BTN     VALUE "Add" ON-ACTION PERFORM HANDLE-ADD.
               10  SUB-BTN     VALUE "Sub" ON-ACTION PERFORM HANDLE-SUB.
               10  MUL-BTN     VALUE "Mul" ON-ACTION PERFORM HANDLE-MUL.
               10  DIV-BTN     VALUE "Div" ON-ACTION PERFORM HANDLE-DIV.
               10  CHECK-BTN   VALUE "Check" ON-ACTION PERFORM HANDLE-CHECK.
           05  STATUS-BAR.
               10  MSG-TEXT    PIC X(60) USING STATUS-MSG.

       PROCEDURE DIVISION.
       MAIN-LOOP.
           STOP RUN.

       HANDLE-ADD.
           MOVE OPERAND-A TO RESULT-VAL.
           ADD OPERAND-B TO RESULT-VAL.
           MOVE "Added" TO STATUS-MSG.

       HANDLE-SUB.
           MOVE OPERAND-A TO RESULT-VAL.
           SUBTRACT OPERAND-B FROM RESULT-VAL.
           MOVE "Subtracted" TO STATUS-MSG.

       HANDLE-MUL.
           MOVE OPERAND-A TO RESULT-VAL.
           MULTIPLY OPERAND-B BY RESULT-VAL.
           MOVE "Multiplied" TO STATUS-MSG.

       HANDLE-DIV.
           IF OPERAND-B = 0
               MOVE "Cannot divide by zero!" TO STATUS-MSG
           ELSE
               MOVE OPERAND-A TO RESULT-VAL
               DIVIDE OPERAND-B INTO RESULT-VAL
               MOVE "Divided" TO STATUS-MSG
           END-IF.

       HANDLE-CHECK.
           IF RESULT-VAL > 1000
               MOVE "Result is large!" TO STATUS-MSG
           ELSE
               IF RESULT-VAL = 0
                   MOVE "Result is zero" TO STATUS-MSG
               ELSE
                   MOVE "Result is modest" TO STATUS-MSG
               END-IF
           END-IF.
