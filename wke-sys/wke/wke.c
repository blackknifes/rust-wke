#include "wke.h"
#include <Windows.h>

void win32RunLoop()
{
    MSG msg;
    while (GetMessage(&msg, NULL, 0, 0)){
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }
}

int win32RunLoopOnce()
{
    MSG msg;
    while (PeekMessage(&msg, NULL, 0, 0, PM_REMOVE)) {
        if (msg.message == WM_QUIT)
        {
            while (PeekMessage(&msg, NULL, 0, 0, PM_REMOVE))
            {
                TranslateMessage(&msg);
                DispatchMessage(&msg);        
            }
            return -1;
        }
        
        TranslateMessage(&msg);
        DispatchMessage(&msg);
        return 1;
    }

    return 0;
}

void win32ExitLoop()
{
    PostQuitMessage(0);
}