       IDENTIFICATION DIVISION.
       PROGRAM-ID. CALCULATOR.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  APP-STATE.
           05  OPERAND-A       PIC 9(4) VALUE 0.
           05  OPERAND-B       PIC 9(4) VALUE 0.
           05  RESULT-VAL      PIC 9(8) VALUE 0.
           05  MEMORY-VAL      PIC 9(8) VALUE 0.
           05  OPERATION        PIC X(10) VALUE "ADD".
               88  IS-ADDING    VALUE "ADD".
               88  IS-SUBBING   VALUE "SUB".
               88  IS-MULTING   VALUE "MUL".
               88  IS-DIVIDING  VALUE "DIV".
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
               10  MEM-DISPLAY PIC 9(8) USING MEMORY-VAL.
           05  OPERATIONS.
               10  ADD-BTN     VALUE "Add" ON-ACTION PERFORM HANDLE-ADD.
               10  SUB-BTN     VALUE "Sub" ON-ACTION PERFORM HANDLE-SUB.
               10  MUL-BTN     VALUE "Mul" ON-ACTION PERFORM HANDLE-MUL.
               10  DIV-BTN     VALUE "Div" ON-ACTION PERFORM HANDLE-DIV.
           05  MEMORY-CONTROLS.
               10  MPLUS-BTN   VALUE "M-Plus" ON-ACTION PERFORM HANDLE-MPLUS.
               10  MMINUS-BTN  VALUE "M-Minus" ON-ACTION PERFORM HANDLE-MMINUS.
               10  MR-BTN      VALUE "MR" ON-ACTION PERFORM HANDLE-MR.
               10  MC-BTN      VALUE "MC" ON-ACTION PERFORM HANDLE-MC.
           05  EXTRA-CONTROLS.
               10  COMPUTE-BTN VALUE "Expr" ON-ACTION PERFORM HANDLE-COMPUTE.
               10  CHECK-BTN   VALUE "Check" ON-ACTION PERFORM HANDLE-CHECK.
           05  STATUS-BAR.
               10  MSG-TEXT    PIC X(60) USING STATUS-MSG.

       PROCEDURE DIVISION.
       MAIN-LOOP.
           STOP RUN.

       HANDLE-ADD.
           SET IS-ADDING TO TRUE.
           PERFORM DO-OPERATION.

       HANDLE-SUB.
           SET IS-SUBBING TO TRUE.
           PERFORM DO-OPERATION.

       HANDLE-MUL.
           SET IS-MULTING TO TRUE.
           PERFORM DO-OPERATION.

       HANDLE-DIV.
           SET IS-DIVIDING TO TRUE.
           PERFORM DO-OPERATION.

       DO-OPERATION.
           EVALUATE OPERATION
               WHEN "ADD"
                   MOVE OPERAND-A TO RESULT-VAL
                   ADD OPERAND-B TO RESULT-VAL
                   MOVE "Added" TO STATUS-MSG
               WHEN "SUB"
                   MOVE OPERAND-A TO RESULT-VAL
                   SUBTRACT OPERAND-B FROM RESULT-VAL
                   MOVE "Subtracted" TO STATUS-MSG
               WHEN "MUL"
                   MOVE OPERAND-A TO RESULT-VAL
                   MULTIPLY OPERAND-B BY RESULT-VAL
                   MOVE "Multiplied" TO STATUS-MSG
               WHEN "DIV"
                   IF OPERAND-B = 0
                       MOVE "Cannot divide by zero!" TO STATUS-MSG
                   ELSE
                       MOVE OPERAND-A TO RESULT-VAL
                       DIVIDE OPERAND-B INTO RESULT-VAL
                       MOVE "Divided" TO STATUS-MSG
                   END-IF
               WHEN OTHER
                   MOVE "Unknown operation" TO STATUS-MSG
           END-EVALUATE.

       HANDLE-COMPUTE.
           COMPUTE RESULT-VAL = OPERAND-A + OPERAND-B * 2.
           MOVE "Computed expression" TO STATUS-MSG.

       HANDLE-MPLUS.
           ADD RESULT-VAL TO MEMORY-VAL.
           MOVE "Memory added" TO STATUS-MSG.

       HANDLE-MMINUS.
           SUBTRACT RESULT-VAL FROM MEMORY-VAL.
           MOVE "Memory subtracted" TO STATUS-MSG.

       HANDLE-MR.
           MOVE MEMORY-VAL TO RESULT-VAL.
           MOVE "Memory recalled" TO STATUS-MSG.

       HANDLE-MC.
           MOVE 0 TO MEMORY-VAL.
           MOVE "Memory cleared" TO STATUS-MSG.

       HANDLE-CHECK.
           EVALUATE RESULT-VAL
               WHEN 0
                   MOVE "Result is zero" TO STATUS-MSG
               WHEN OTHER
                   IF RESULT-VAL > 1000
                       MOVE "Result is large!" TO STATUS-MSG
                   ELSE
                       MOVE "Result is modest" TO STATUS-MSG
                   END-IF
           END-EVALUATE.
