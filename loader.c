//
// Copyright (C) 2017 Tavis Ormandy
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//

#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <ctype.h>
#include <stdarg.h>
#include <assert.h>
#include <string.h>
#include <time.h>
#include <sys/resource.h>
#include <sys/unistd.h>
#include <asm/unistd.h>
#include <sys/types.h>
#include <sys/time.h>
#include <sys/stat.h>
#include <sys/mman.h>
#include <signal.h>
#include <fcntl.h>
#include <unistd.h>
#include <mcheck.h>
#include <err.h>

#include "winnt_types.h"
#include "pe_linker.h"
#include "ntoskernel.h"
#include "util.h"
#include "hook.h"
#include "log.h"
#include "rsignal.h"
#include "engineboot.h"
#include "scanreply.h"
#include "streambuffer.h"
#include "openscan.h"

#include "AILib.h"
#include "MIPIface.h"

typedef struct
{
    UINT hai[18];
    int num;
} MJ_SUTEHAI;

typedef struct
{
    UINT tehai[13];
    MJ_SUTEHAI sutehai[4];
    UINT dora[4];
    int doranum;
    int tehai_max;
    UINT tsumohai;
    int kyoku;
    int zikaze;
} MJ_TAKU;

static MJ_TAKU taku = {
    {6, 6, 8, 8, 8, 13, 14, 14, 15, 20, 20, 22, 24},
    {
        {{32, 8, 29, 27, 31, 10}, 6},
        {{17, 28, 7, 10, 2, 2}, 6},
        {{32, 29, 18, 30, 11, 27}, 6},
        {{18, 32, 29, 30, 31, 28, 17}, 7},
    },
    {0},
    1,
    13,
    25,
    7,
    1};

UINT WINAPI MJPInterfaceFunc(void *inst, UINT message, UINT param1, UINT param2);

typedef struct
{
    GAMESTATE gamestate;
    int agarihai;
} MJ_GAMESTATE;

static int scoreCallback(int *paiarray, int *mentsu, int length, int machi, void *inf)
{
    RESULT_ITEM item;
    MJ_GAMESTATE *state = (MJ_GAMESTATE *)inf;
    make_resultitem(paiarray, mentsu, length, &item, &state->gamestate, state->agarihai, machi);

    return item.score;
    // return item.mentsusize == 7 ? item.score/300 : item.score;
}

UINT WINAPI MJSendMessage(void *inst, UINT message, UINT param1, UINT param2)
{
    UINT ret = 0;
    int idx;
    int i, j;
    MJITehai *pTehai;
    UINT *p;
    MJ_SUTEHAI *pSutehai;
    MJIKawahai *pKawahai;
    printf("message flag = %08x param1 = %08x param2 = %08x \n", message, param1, param2);

    switch (message)
    {
    case MJMI_GETTEHAI:
        pTehai = (MJITehai *)param2;
        memset(pTehai, 0, sizeof(MJITehai));
        if (param1 == 0)
        {
            memcpy(pTehai->tehai, taku.tehai, taku.tehai_max * sizeof(UINT));
            pTehai->tehai_max = taku.tehai_max;
        }
        ret = 1;
        break;
    case MJMI_GETMACHI:
        p = (UINT *)param2;
        for (i = 0; i < 34; i++)
        {
            p[i] = 0;
        }
        ret = 0;
        break;
    case MJMI_GETAGARITEN:
        if (param1 != 0)
        {
            ret = 100;
        }
        break;
    case MJMI_GETKAWA:
        idx = LOWORD(param1);
        memcpy((UINT *)param2, &taku.sutehai[idx].hai, sizeof(UINT) * taku.sutehai[idx].num);
        ret = taku.sutehai[idx].num;
        break;
    case MJMI_GETKAWAEX:
        idx = LOWORD(param1);
        pKawahai = (MJIKawahai *)param2;
        pSutehai = &taku.sutehai[idx];
        for (i = 0; i < pSutehai->num; i++)
        {
            pKawahai[i].hai = pSutehai->hai[i];
            pKawahai[i].state = 0;
        }
        ret = pSutehai->num;
        break;
    case MJMI_GETDORA:
        p = (UINT *)param1;
#if 0
		for(i=0;i<taku.doranum;i++){
				p[i] = taku.dora[i];
		}
#else
        for (i = 0; i < taku.doranum; i++)
        {
            switch (taku.dora[i])
            {
            case 33:
                p[i] = 31;
                break;
            case 30:
                p[i] = 27;
                break;
            case 8:
                p[i] = 0;
                break;
            case 17:
                p[i] = 9;
                break;
            case 26:
                p[i] = 18;
                break;
            default:
                p[i] = taku.dora[i] + 1;
                break;
            }
        }
#endif
        ret = taku.doranum;
        break;
    case MJMI_GETHAIREMAIN:
        ret = 70;
        for (i = 0; i < 4; i++)
        {
            ret -= taku.sutehai[i].num;
        }
        ret--;
        break;
    case MJMI_GETVISIBLEHAIS:
        ret = 0;
        idx = LOWORD(param1);

        for (i = 0, pSutehai = &taku.sutehai[0]; i < 4; i++, pSutehai++)
        {
            for (j = 0; j < pSutehai->num; j++)
            {
                if (pSutehai->hai[j] == idx)
                    ret++;
            }
        }

        for (i = 0; i < taku.doranum; i++)
        {
            if (taku.dora[i] == idx)
            {
                ret++;
            }
        }

        break;
    case MJMI_FUKIDASHI:
        printf((const char *)param1);
        printf("\n");
        break;
    case MJMI_GETSCORE:
        ret = 25000;
        break;
    case MJMI_GETVERSION:
        ret = 12;
        break;
    case MJMI_GETKYOKU:
    case MJMI_GETHONBA:
    case MJMI_GETREACHBOU:
    case MJMI_ANKANABILITY:
    case MJMI_KKHAIABILITY:
    case MJMI_LASTTSUMOGIRI:
    case MJMI_GETRULE:
    case MJMI_SETSTRUCTTYPE:
    case MJMI_SETAUTOFUKIDASHI:
    case MJMI_GETWAREME:
    default:
        ret = 0;
        break;
    }

    return ret;
}

int main(int argc, char **argv, char **envp)
{
    MJPIFunc func;
    void *inst;
    UINT ret;

    if (argc < 2)
    {
        LogMessage("usage: %s [filenames...]", *argv);
        return 1;
    }

    for (char *filename = *++argv; *argv; ++argv)
    {
        LogMessage("Scanning %s...", *argv);
        struct pe_image image = {
            .entry = NULL,
        };

        strcpy(image.name, *argv);

        // Load the mpengine module.
        if (pe_load_library(image.name, &image.image, &image.size) == false)
        {
            LogMessage("You must add the dll and vdm files to the engine directory");
            return 1;
        }

        // Handle relocations, imports, etc.
        link_pe_images(&image, 1);

        printf("entry : %p\n", image.entry);

        // Call DllMain()
        image.entry((PVOID)'MPEN', DLL_PROCESS_ATTACH, NULL);

        if (get_export("MJPInterfaceFunc", &func) != 0)
        {
            LogMessage("Cannot load MJPInterfaceFunc!");
            return 1;
        }

        size_t size = func(NULL, MJPI_CREATEINSTANCE, 0, 0);

        size = func(NULL, MJPI_CREATEINSTANCE, 0, 0);
        if (size > 0)
        {
            inst = malloc(size);
            func(inst, MJPI_INITIALIZE, 0, (UINT)MJSendMessage);
            /* 途中参加でエミュレート */
            func(inst, MJPI_ONEXCHANGE, MJST_INKYOKU, MAKELPARAM(taku.kyoku, taku.zikaze));

            ret = func(inst, MJPI_SUTEHAI, taku.tsumohai, 0);

            printf("ret = %d flag = %04x\n", ret & 0x3F, ret & 0xFF80);

            free(inst);
        }

        return 0;
    }
}
