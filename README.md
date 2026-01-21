# Image Cropper

A simple and efficient image cropping tool written in Rust using [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) (egui framework). It provides an intuitive interface for cropping images with support for various aspect ratios.

## Features

*   **Easy Image Loading**: Open images via the "Open Image" button or simply **Drag & Drop** files into the window.
*   **Format Support**: Supports common image formats including PNG, JPG/JPEG, and BMP.
*   **Aspect Ratio Control**:
    *   **Presets**: Includes popular aspect ratios like 16:9, 16:10, 4:3, 3:2, and 1:1 (Square).
    *   **Orientation**: Quickly toggle between Landscape and Portrait modes using the Rotate button (🔄).
    *   **Original**: Lock to the original image's aspect ratio.
    *   **Custom**: Define your own width and height ratios.
    *   **Free**: Unconstrained freeform cropping.
*   **Visual Guides**: dimmed overlay showing the area to be cropped out.
*   **Interactive Cropping**: Resize handles (corners and sides) and center-drag to move the crop area.
*   **Cross-Platform**: Runs on macOS, Windows, and Linux (powered by Rust and egui).

## Installation

### Download

You can download the latest pre-built binaries for macOS, Windows, and Linux from the [Releases](https://github.com/yutingli1123/ImageCropper/releases) page.

### Prerequisites

You need to have **Rust** installed on your system. If you haven't installed it yet, you can get it from [rustup.rs](https://rustup.rs/).

### Build and Run

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/yutingli1123/ImageCropper.git
    cd ImageCropper
    ```

2.  **Run the application:**
    ```bash
    cargo run --release
    ```

## Usage

1.  Launch the application.
2.  Click **Open Image** or drop an image file onto the window.
3.  Choose your desired **Aspect Ratio** from the dropdown menu.
    *   Use the **Rotate button (🔄)** to swap dimensions (e.g., 4:3 ↔ 3:4).
    *   Select **Custom** to enter specific ratio values.
4.  Adjust the crop rectangle by dragging the corners, sides, or the rectangle itself.
5.  Click **Save Cropped Image** to save your result to disk.
