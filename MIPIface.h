/* �܂������ �ΐ푊��p�}�N�� **
**         version 12          **
**                 �Δ� ����   */
#ifndef __MIPIFACE_H__
#define __MIPIFACE_H__

typedef unsigned UINT;
typedef unsigned short USHORT;

#define MJ_INTERFACE_VERSION 12

/* Messages for player's interface */
#define MJPI_INITIALIZE 1
#define MJPI_SUTEHAI 2
#define MJPI_ONACTION 3
#define MJPI_STARTGAME 4
#define MJPI_STARTKYOKU 5
#define MJPI_ENDKYOKU 6
#define MJPI_ENDGAME 7
#define MJPI_DESTROY 8
#define MJPI_YOURNAME 9
#define MJPI_CREATEINSTANCE 10
#define MJPI_BASHOGIME 11
#define MJPI_ISEXCHANGEABLE 12
#define MJPI_ONEXCHANGE 13

/* Messages for system interface */
#define MJMI_GETTEHAI 1
#define MJMI_GETKAWA 2
#define MJMI_GETDORA 3
#define MJMI_GETSCORE 4
#define MJMI_GETHONBA 5
#define MJMI_GETREACHBOU 6
#define MJMI_GETRULE 7
#define MJMI_GETVERSION 8
#define MJMI_GETMACHI 9
#define MJMI_GETAGARITEN 10
#define MJMI_GETHAIREMAIN 11
#define MJMI_GETVISIBLEHAIS 12
#define MJMI_FUKIDASHI 13
#define MJMI_KKHAIABILITY 14
#define MJMI_GETWAREME 15
#define MJMI_SETSTRUCTTYPE 16
#define MJMI_SETAUTOFUKIDASHI 17
#define MJMI_LASTTSUMOGIRI 18
#define MJMI_SSPUTOABILITY 19
#define MJMI_GETYAKUHAN 20
#define MJMI_GETKYOKU 21
#define MJMI_GETKAWAEX 22
#define MJMI_ANKANABILITY 23

/* Macro */
#define MJPIR_SUTEHAI 0x00000100
#define MJPIR_REACH 0x00000200
#define MJPIR_KAN 0x00000400
#define MJPIR_TSUMO 0x00000800
#define MJPIR_NAGASHI 0x00001000
#define MJPIR_PON 0x00002000
#define MJPIR_CHII1 0x00004000
#define MJPIR_CHII2 0x00008000
#define MJPIR_CHII3 0x00010000
#define MJPIR_MINKAN 0x00020000
#define MJPIR_ANKAN 0x00040000
#define MJPIR_RON 0x00080000

#define MJMIR_ERROR 0x80000000
#define MJR_NOTCARED 0xffffffff

/* RULE Macro */
#define MJRL_KUITAN 1
#define MJRL_KANSAKI 2
#define MJRL_PAO 3
#define MJRL_RON 4
#define MJRL_MOCHITEN 5
#define MJRL_BUTTOBI 6
#define MJRL_WAREME 7
#define MJRL_AKA5 8
#define MJRL_SHANYU 9
#define MJRL_SHANYU_SCORE 10
#define MJRL_KUINAOSHI 11
#define MJRL_AKA5S 12
#define MJRL_URADORA 13
#define MJRL_SCORE0REACH 14
#define MJRL_RYANSHIBA 15
#define MJRL_DORAPLUS 16
#define MJRL_FURITENREACH 17
#define MJRL_NANNYU 18
#define MJRL_NANNYU_SCORE 19
#define MJRL_KARATEN 20
#define MJRL_PINZUMO 21
#define MJRL_NOTENOYANAGARE 22
#define MJRL_KANINREACH 23
#define MJRL_TOPOYAAGARIEND 24
#define MJRL_77MANGAN 25
#define MJRL_DBLRONCHONBO 26

/* YAKU macros */
enum MJI_YAKU
{
    MJYK_REACH = 0,
    MJYK_IPPATSU,
    MJYK_PINFU,
    MJYK_IPEKO,
    MJYK_TANYAO,
    MJYK_FANPAI,
    MJYK_TSUMO,
    MJYK_HAITEI,
    MJYK_HOTEI,
    MJYK_RINSHAN,
    MJYK_CHANKAN,
    MJYK_DOUBLEREACH,
    MJYK_CHITOI,
    MJYK_CHANTA,
    MJYK_ITTSU,
    MJYK_SANSHOKUDOUJUN,
    MJYK_SANSHOKUDOUKOU,
    MJYK_TOITOI,
    MJYK_SANANKOU,
    MJYK_SANKANTSU,
    MJYK_SHOUSANGEN,
    MJYK_HONROUTOU,
    MJYK_SANRENKOU,
    MJYK_RYANPEKO,
    MJYK_HONITSU,
    MJYK_JUNCHAN,
    MJYK_CHINITSU,
    MJYK_RENHO,
    MJYK_TENHO,
    MJYK_CHIHO,
    MJYK_DAISANGEN,
    MJYK_TSUISO,
    MJYK_SUSHIHO,
    MJYK_CHINROUTOU,
    MJYK_SUKANTSU,
    MJYK_RYUISO,
    MJYK_SUANKOU,
    MJYK_KOKUSHI,
    MJYK_CHUREN,
    MJYK_SISANPUTO,
    MJYK_DAISHARIN,
    MJYK_NAGASHIMANGAN,
    MJYK_DORA,
    MJYK_SURENKOU,
    MJYK_ISSHOKUSANJUN,
    MJYK_ISSHOKUYONJUN,
    MJI_YAKUS
};

/* REASON of END KYOKU */
#define MJEK_AGARI 1
#define MJEK_RYUKYOKU 2
#define MJEK_CHONBO 3

/* STATE of Game */
#define MJST_INKYOKU 1
#define MJST_BASHOGIME 2

/* TEHAI Structure */
typedef struct
{
    UINT tehai[14];
    UINT tehai_max;
    UINT minshun[4];
    UINT minshun_max;
    UINT minkou[4];
    UINT minkou_max;
    UINT minkan[4];
    UINT minkan_max;
    UINT ankan[4];
    UINT ankan_max;
    UINT reserved1;
    UINT reserved2;
} MJITehai;

typedef MJITehai MJITehai0;

typedef struct
{
    unsigned hai_no : 6;
    unsigned aka : 1;
} MJIHai;

typedef struct
{
    UINT tehai[14];
    UINT tehai_max;
    UINT minshun[4];
    UINT minshun_max;
    UINT minkou[4];
    UINT minkou_max;
    UINT minkan[4];
    UINT minkan_max;
    UINT ankan[4];
    UINT ankan_max;
    UINT minshun_hai[3][4];
    UINT minkou_hai[3][4];
    UINT minkan_hai[4][4];
    UINT ankan_hai[4][4];
    UINT reserved1;
    UINT reserved2;
} MJITehai1;

/* Expression of KAWAHAI */
typedef struct
{
    USHORT hai;
    USHORT state;
} MJIKawahai;

#define MJKS_REACH 1
#define MJKS_NAKI 2

typedef UINT(WINAPI *MJPIFunc)(void *, UINT, UINT, UINT);

#define LOWORD(x) (x & 0xFFFF)
#define MAKELPARAM(l, r) (l << 16 | (r & 0xFFFF))

#endif
