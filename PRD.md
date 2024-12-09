# Product Requirements Document
AFTER EVERY CHANGE, UPDATE THE StatusToDoComplete.md file.
## Overview
This document defines the requirements for a highly efficient, adaptive screen sharing platform. The platform dynamically adjusts quality and bitrate to maintain smooth streaming at up to 60fps (targeting a stable 30fps under typical conditions). The system incorporates a PixelChangeCheck (PCC) framework to minimize redundant data transmission, ensuring optimal performance even in low-bandwidth scenarios. The platform is intended for real-time interaction, supports localhost testing, and maintains frame accuracy on the receiving end.

## Goals and Objectives
- Deliver a screen sharing solution that uses minimal bandwidth without sacrificing visual clarity.
- Achieve near real-time latency and up to 60fps streaming, with a consistent 30fps target.
- Utilize PCC to send only changed pixels between frames, significantly reducing unnecessary data transmission.
- Allow localhost testing for development and debugging.
- Provide a scalable foundation that can be easily integrated into various hosting environments.

## Target Use Cases
1. Remote desktop sharing where user interfaces remain largely static for extended periods.
2. Presentations or slideshows where content updates infrequently.
3. Embedded dynamic elements (e.g., embedded videos in a webpage) where only changed regions are updated.
4. Low-bandwidth conditions where selective pixel updates ensure a smooth experience.

## User Flows
1. **Initiating a Session**: The client starts capturing its screen and establishes a connection to the server. The server awaits incoming data.
2. **Steady State Transmission**: 
   - On each frame, the client uses PCC to detect changed pixels compared to the previous frame.
   - The client transmits only those changed pixels to the server.
   - The server updates its displayed frame accordingly.
3. **Idle State**: 
   - If no pixels change, the client sends periodic keep-alive signals rather than frame data.
   - The server continues to render the last received full frame.
4. **Low-Bandwidth Adaptation**: 
   - If bandwidth decreases, the system reduces frame rate or resolution incrementally.
   - If conditions improve, it can ramp back up to higher quality or framerates.
5. **Ending the Session**: The client sends a termination signal, and the server stops displaying new frames, ending the session.

## PixelChangeCheck (PCC) Details
- PCC compares the current frame’s pixel data against the previous frame’s pixel data on the client side.
- If no changes are detected, no new frame data is sent. The only communication is a keep-alive signal.
- If partial changes are detected (e.g., a small UI element updates), only that subset of pixels is transmitted.
- This approach ensures minimal data overhead while preserving visual fidelity.

## Functional Requirements
1. **Frame Capture**: The client must capture full frames up to 60fps and generate a pixel-difference map.
2. **PCC Implementation**: The client must compute pixel-level diffs efficiently, minimizing latency.
3. **Adaptive Quality**: The client must dynamically adjust bitrate and resolution based on real-time network conditions.
4. **Partial Updates**: The client must transmit only changed pixel data and keep-alives if no changes occur.
5. **Server Rendering**: The server must rebuild frames from the partial updates and hold the last known frame if no changes arrive.
6. **Keep-Alive Signals**: The client must send periodic keep-alives to indicate that the session remains active.
7. **Localhost Testing**: The system must run in a localhost environment for development and quality assurance.

## Non-Functional Requirements
1. **Latency**: Round-trip latency should be low enough for real-time interaction.
2. **Scalability**: The architecture should allow multiple concurrent sessions without performance degradation.
3. **Reliability**: The server must continue displaying the last known frame if the client is momentarily idle.
4. **Resource Utilization**: CPU and memory usage should remain manageable, even at higher frame rates.

## Architecture Overview
- **Client Component**: 
  - Captures screens at intervals.
  - Identifies changed pixels using PCC.
  - Encodes and transmits only the updated regions or keep-alives.
  - Adjusts bitrate and quality dynamically.
- **Server Component**:
  - Receives partial frame updates or keep-alives from the client.
  - Updates the displayed frame buffer accordingly.
  - Maintains the last known frame if no new data arrives.
  
## Testing and Validation
- Localhost testing to verify PCC logic, ensuring unchanged frames incur no data overhead.
- Stress tests simulating varying network conditions and resource availability.
- Performance tests evaluating latency and frame rates under different scenarios.
  
## Milestones
1. **Prototype with Basic PCC**: Initial implementation sending full frames and diff-based updates.
2. **Adaptive Quality Mechanism**: Dynamic bitrate and resolution adjustments.
3. **Stability and Idle Mode**: Keep-alive signals and prolonged idle state handling.
4. **Polish and Optimization**: Further CPU, memory optimizations, and network protocol refinements.

## Future Considerations
- Adding encryption for secure transmission.
- Extending PCC logic for multiple viewers or broadcast scenarios.
- Integrating adaptive codecs for better compression efficiency.


## Original PRD with My Examples
- Overview:
An extremely efficient screen sharing platform/protocol, like airplay, auto adjusting quality(up to 60fps, goal 30fps) and bitrate, etc, that uses PixelChanceCheck(PCC) to prevent unnecessary data transmission. (client is the transmitter, server is the viewer). This should be able to be localhost tested.

- PixelChangeCheck:
PCC is a framework that compares the current frame to the last frame and only sends the data if it has changed. For example, if someone is staying on an unmoving page, the client WON'T send new data, until some pixels are changed. The server will keep rendering the last frame given, indefinitely. The client will send periodic “keep-alive” requests to the server, telling it that the connection has not ended. If only small amounts of pixels are changed, it will only resend those pixels/data for the server to change, For example, if a small button changed color, it will only resend the data for the changed pixels. This expands to larger parts. For example an embedded youtube video in a slideshow. It will keep the background of the slideshow, and maybe even some parts of the video that haven't changed, but it will send the new/changed parts of the video. 

AFTER EVERY CHANGE, UPDATE THE StatusToDoComplete.md file.
