import pyautogui
import time

print("Press Ctrl+C to stop the program.")

time.sleep(3)  # gives you time to switch to the target window

while True:
    pyautogui.hotkey('alt', 'enter')
    print("Pressed Alt + Enter")
    time.sleep(10)

