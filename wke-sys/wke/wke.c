#include "wke.h"
#include <Windows.h>

void rustRunLoop()
{
    MSG msg;
    while (GetMessage(&msg, NULL, 0, 0)){
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }
}

int rustRunLoopOnce()
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

void rustExitLoop()
{
    PostQuitMessage(0);
}