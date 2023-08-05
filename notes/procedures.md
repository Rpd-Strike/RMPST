---------------------------------------
Types of communication

Hist:
 - Send confirmation:
   - receive send tag
   - send back the tag confirmation

 - Rollback
   - receive rollback notification
   - send back freeze notifications

 - Dissapear
   - receive dissapear notification from processes who want to rolback one process for one step
   - send resurrect messages to the previous sender & receiver

Part:
 - Evolve state:
   - End
     - nothing
   - Rollback
     - Send rollback notification to Hist
   - Send
     - Send payload on the specified channel with owner information
   - Recv
     - Poll all receive processes
     - after identifying a receive channel, send new tag to Hist
     - Receive tag confirmation from Hist
     - Perform alpha conversions and add to active PrimeState

 - Freeze notification
   - Receive freeze notifications from Hist
   - Eagerly try to mark live processes as frozen

 - Dissapear
   - For every frozen live process, send a dissapear notification and eliminate from live state
 
 - Resurrect
   - Wait for resurrect messages
   - For every resurrect, append the process to the live list

--------------------------------
Procedures:

| Normal communication rule
  - Part sends to another part the payload and who he is
  - Part recv and sends a memory piece to Hist
  - Hist receives the memory piece, updates records and sends confirmation
  - Part recv waits for confirmation message from Hist

| Rolback
  - Part sends a rollback message
  - Hist receives a rollback message, 
  - If the rolled message is not already marked as frozen, then start marking reursively and send freeze notification

| Freeze notification
  - Hist sends freeze notification
  - Part receives freeze notification
  - Part tries to eagerly mark more frozen tags (in practice, tries to mark the live processes)
  - Start Dissapear & Resurrect procedure

| Dissapear & Resurrect procedure
  - Part: For all frozen live processes, send a Dissapear notification to Hist
  - Hist: Receives a Dissapear notification
  - Hist: Send the receiver and sender side processes to the respective owners with a Resurrect msg

------------------------------------
Chanel stores
| Communication
  - Each participant has senders + receivers for all channel names

| Comm confirmation
  - Each participant has the same Sender for sending tag
  - History has the Receiver for the sending tag channel

  - Each participant has its own receiver for confirmation
  - History has a map for sending confirmation for each participant