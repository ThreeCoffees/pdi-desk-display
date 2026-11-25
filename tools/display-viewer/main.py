
import threading
import tkinter as tk
from tkinter import ttk, messagebox
from urllib.request import urlopen, Request


def decode_framebuffer(data: bytes, width: int, height: int):
    expected = (width * height + 7) // 8

    if len(data) != expected:
        raise ValueError(
            f"Invalid framebuffer size: {len(data)} bytes. "
            f"Expected {expected} bytes."
        )

    # Tkinter PPM image: RGB, 8 bits per channel.
    pixels = bytearray(width * height * 3)

    for i in range(width * height):
        byte = data[i // 8]
        bit = (byte >> (7 - (i % 8))) & 1

        # 1 = white, 0 = black
        value = 255 if bit else 0

        pixels[i * 3:i * 3 + 3] = bytes([value, value, value])

    return (
        f"P6\n{width} {height}\n255\n".encode() + pixels
    )


class EpaperViewer:
    def __init__(self, root):
        self.root = root
        self.root.title("E-paper Viewer")
        self.root.geometry("900x650")

        self.url = tk.StringVar(value="http://192.168.4.1/display")
        self.width = tk.StringVar(value="800")
        self.height = tk.StringVar(value="480")
        self.status = tk.StringVar(value="Ready")

        controls = ttk.Frame(root, padding=10)
        controls.pack(fill="x")

        ttk.Label(controls, text="Endpoint:").grid(row=0, column=0)
        ttk.Entry(
            controls, textvariable=self.url, width=45
        ).grid(row=0, column=1, columnspan=4, sticky="ew", padx=5)

        ttk.Label(controls, text="Width:").grid(row=1, column=0, pady=10)
        ttk.Entry(
            controls, textvariable=self.width, width=8
        ).grid(row=1, column=1, sticky="w")

        ttk.Label(controls, text="Height:").grid(row=1, column=2)
        ttk.Entry(
            controls, textvariable=self.height, width=8
        ).grid(row=1, column=3, sticky="w")

        self.refresh_button = ttk.Button(
            controls,
            text="Refresh screen",
            command=self.refresh
        )
        self.refresh_button.grid(row=1, column=4, padx=10)

        controls.columnconfigure(1, weight=1)

        preview = ttk.Frame(root, padding=10)
        preview.pack(fill="both", expand=True)

        self.canvas = tk.Canvas(preview, background="#777777")

        scrollbar_x = ttk.Scrollbar(
            preview, orient="horizontal",
            command=self.canvas.xview
        )
        scrollbar_y = ttk.Scrollbar(
            preview, orient="vertical",
            command=self.canvas.yview
        )

        self.canvas.configure(
            xscrollcommand=scrollbar_x.set,
            yscrollcommand=scrollbar_y.set
        )

        self.canvas.grid(row=0, column=0, sticky="nsew")
        scrollbar_y.grid(row=0, column=1, sticky="ns")
        scrollbar_x.grid(row=1, column=0, sticky="ew")

        preview.rowconfigure(0, weight=1)
        preview.columnconfigure(0, weight=1)

        ttk.Label(
            root, textvariable=self.status, padding=8
        ).pack(fill="x")

        self.image = None
        self.image_item = None

    def refresh(self):
        try:
            width = int(self.width.get())
            height = int(self.height.get())

            if width <= 0 or height <= 0:
                raise ValueError("Dimensions must be positive.")

            if width * height > 20_000_000:
                raise ValueError("Image is too large.")

            url = self.url.get().strip()

            if not url.startswith(("http://", "https://")):
                raise ValueError("Invalid endpoint URL.")

        except ValueError as error:
            messagebox.showerror("Invalid input", str(error))
            return

        self.refresh_button.configure(state="disabled")
        self.status.set("Fetching framebuffer...")

        threading.Thread(
            target=self.fetch,
            args=(url, width, height),
            daemon=True
        ).start()

    def fetch(self, url, width, height):
        try:
            request = Request(
                url,
                headers={"Accept": "application/octet-stream"}
            )

            with urlopen(request, timeout=10) as response:
                data = response.read()

            ppm = decode_framebuffer(data, width, height)

            self.root.after(
                0, lambda: self.show_image(ppm, width, height)
            )

        except Exception as error:
            message = str(error)
            self.root.after(
                0, lambda msg=message: self.show_error(msg)
            )

    def show_image(self, ppm, width, height):
        try:
            self.image = tk.PhotoImage(data=ppm, format="PPM")

            if self.image_item is None:
                self.image_item = self.canvas.create_image(
                    0, 0, anchor="nw", image=self.image
                )
            else:
                self.canvas.itemconfigure(
                    self.image_item, image=self.image
                )

            self.canvas.configure(
                scrollregion=(0, 0, width, height)
            )

            self.status.set(
                f"Received framebuffer: {width} × {height}"
            )

        except tk.TclError as error:
            self.show_error(str(error))
            return

        self.refresh_button.configure(state="normal")

    def show_error(self, error):
        self.status.set("Request failed")
        self.refresh_button.configure(state="normal")
        messagebox.showerror("Error", error)


if __name__ == "__main__":
    root = tk.Tk()
    EpaperViewer(root)
    root.mainloop()

