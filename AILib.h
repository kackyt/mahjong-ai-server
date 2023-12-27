/****************************************************************************************
 * Copyright (c) 2010, Takaya Kakizaki(kacky)
 * All rights reserved.

  �\�[�X�R�[�h�`�����o�C�i���`�����A�ύX���邩���Ȃ������킸�A�ȉ��̏����𖞂����ꍇ�Ɍ���A�ĔЕz����юg�p��������܂��B

  �E�\�[�X�R�[�h���ĔЕz����ꍇ�A��L�̒��쌠�\���A�{�����ꗗ�A����щ��L�Ɛӏ������܂߂邱�ƁB

  �E�o�C�i���`���ōĔЕz����ꍇ�A�Еz���ɕt���̃h�L�������g���̎����ɁA��L�̒��쌠�\���A�{�����ꗗ�A����щ��L�Ɛӏ������܂߂邱�ƁB

  �E���ʂɂ����ʂ̋��Ȃ��ɁA�{�\�t�g�E�F�A����h���������i�̐�`�܂��͔̔����i�ɁA�I�[�v�������̖��O�܂��̓R���g���r���[�^�[�̖��O���g�p���Ă͂Ȃ�Ȃ��B


  �{�\�t�g�E�F�A�́A���쌠�҂���уR���g���r���[�^�[�ɂ���āu����̂܂܁v�񋟂���Ă���A�����َ����킸�A
  ���ƓI�Ȏg�p�\���A����ѓ���̖ړI�ɑ΂���K�����Ɋւ���Öق̕ۏ؂��܂߁A�܂�����Ɍ��肳��Ȃ��A�����Ȃ�ۏ؂�����܂���B
  ���쌠�҂��R���g���r���[�^�[���A���R�̂�������킸�A ���Q�����̌�����������킸�A���ӔC�̍������_��ł��邩���i�ӔC�ł��邩
  �i�ߎ����̑��́j�s�@�s�ׂł��邩���킸�A���ɂ��̂悤�ȑ��Q����������\����m�炳��Ă����Ƃ��Ă��A�{�\�t�g�E�F�A�̎g�p�ɂ���Ĕ�������
  �i��֕i�܂��͑�p�T�[�r�X�̒��B�A�g�p�̑r���A�f�[�^�̑r���A���v�̑r���A�Ɩ��̒��f���܂߁A�܂�����Ɍ��肳��Ȃ��j
  ���ڑ��Q�A�Ԑڑ��Q�A�����I�ȑ��Q�A���ʑ��Q�A�����I���Q�A�܂��͌��ʑ��Q�ɂ��āA��ؐӔC�𕉂�Ȃ����̂Ƃ��܂��B

****************************************************************************************/
#pragma once

#ifdef __cplusplus
extern "C"
{
#endif

#define AI_TEHAI_LIMIT (14)
#define AI_FLAG_NONE (0)
#define AI_FLAG_MACHI (8)          // �҂��v
#define AI_FLAG_EFFECT_SYUNTSU (7) // �L���v
#define AI_FLAG_EFFECT_KOUTSU (6)  // �L���v
#define AI_FLAG_EFFECT_KANTSU (5)  // �L���v
#define AI_FLAG_EFFECT_RYANTAH (4) // �L���v
#define AI_FLAG_EFFECT_ATAMA (3)   // �L���v
#define AI_FLAG_EFFECT_KANTAH (2)  // �L���v
#define AI_FLAG_EFFECT_PENTAH (1)  // �L���v

    typedef struct
    {
        int tehai[14];
        int tehai_max;
    } AGARI_LIST;

    typedef struct
    {
        int mentsuflag[14];
        int machi[34];
        int shanten;
    } TENPAI_LIST;

    typedef struct
    {
        int pai;
        int count;
        int startpos;
    } PAICOUNT;

    typedef struct
    {
        int category;
        int pailist[3];
    } t_mentsu;

    typedef struct
    {
        t_mentsu mentsulist[7];
        int mentsusize;
        int han;
        int fu;
        int score;
        int oyascore;
        int coscore;
        int machipos;
        int machi;
        int machihai;
        int menzen;
    } RESULT_ITEM;

    typedef struct
    {
        t_mentsu nakilist[4];
        int dorapai[8];
        int dorasize;
        int naki;
        int count;
        int tsumo;
        int oya;
        int riichi;
        int bakaze;
        int zikaze;
        int endpai;
        int kan;
        int riichicount;
        int membernaki;
    } GAMESTATE;

#define AI_UKIHAI (0)
#define AI_KOUTSU (1)
#define AI_SYUNTSU (2)
#define AI_SYUNTSU_START (2)
#define AI_SYUNTSU_MIDDLE (3)
#define AI_SYUNTSU_END (4)
#define AI_KANTSU (5)
#define AI_ATAMA (6)
#define AI_TOITSU (7)
#define AI_ANKAN (8)
#define AI_MINKAN (9)
#define AI_MENTSU_LIMIT (10)

#define AI_NIL_PAI (63)

#define AI_MACHI_RYANMEN (0)
#define AI_MACHI_PENCHAN (1)
#define AI_MACHI_KANCHAN (2)
#define AI_MACHI_SHANPON (3)
#define AI_MACHI_TANKI (4)

#define AI_TRUE (1)
#define AI_FALSE (0)

    extern int search_agari(int *paiarray, int paiSize, AGARI_LIST *pList, int actualPaiSize, void *inf, int (*getPoint)(AGARI_LIST *, void *));
    extern int search_tenpai(int *paiarray, int paiSize, int *pMachi, TENPAI_LIST *pList, int listSize, int maxshanten);
    extern int search_score(int *paiarray, int paiSize, void *inf, int (*callback)(int *paiarray, int *mentsu, int length, int machi, void *inf));
    extern void make_resultitem(int *paiarray, int *mentsu, int length, RESULT_ITEM *item, GAMESTATE *gamestate, int agarihai, int machi);
    extern void make_resultitem_bh(RESULT_ITEM *item, GAMESTATE *gamestate);
    extern void tehai_diff_fromcount(const unsigned *tecount_now, unsigned *tecount_future, int n);
    extern void tehai_diff(const unsigned *tehai_now, const unsigned *tehai_future, unsigned *result);
    extern int paidistance(const unsigned *tehai, unsigned pai);

    extern double probabilityFunction(double, int);
    extern int permutation(int m, int n);
    extern int factorial(int n);
    extern int combination(int m, int n);

#ifdef __cplusplus
}
#endif