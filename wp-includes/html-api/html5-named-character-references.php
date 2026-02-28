<?php
require_once dirname( __DIR__ ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/html-api/html5-named-character-references.php' );
if ( '/wp-includes/html-api/html5-named-character-references.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}


/**
 * Auto-generated class for looking up HTML named character references.
 *
 * ⚠️ !!! THIS ENTIRE FILE IS AUTOMATICALLY GENERATED !!! ⚠️
 * Do not modify this file directly.
 *
 * To regenerate, run the generation script directly.
 *
 * Example:
 *
 *     php tests/phpunit/data/html5-entities/generate-html5-named-character-references.php
 *
 * @package WordPress
 * @since 6.6.0
 */

// phpcs:disable

global $html5_named_character_references;

/**
 * Set of named character references in the HTML5 specification.
 *
 * This list will never change, according to the spec. Each named
 * character reference is case-sensitive and the presence or absence
 * of the semicolon is significant. Without the semicolon, the rules
 * for an ambiguous ampersand govern whether the following text is
 * to be interpreted as a character reference or not.
 *
 * The list of entities is sourced directly from the WHATWG server
 * and cached in the test directory to avoid needing to download it
 * every time this file is updated.
 *
 * @link https://html.spec.whatwg.org/entities.json.
 */
$html5_named_character_references = WP_Token_Map::from_precomputed_table(
	array(
		"storage_version" => "6.6.0-trunk",
		"key_length" => 2,
		"groups" => "AE\x00AM\x00Aa\x00Ab\x00Ac\x00Af\x00Ag\x00Al\x00Am\x00An\x00Ao\x00Ap\x00Ar\x00As\x00At\x00Au\x00Ba\x00Bc\x00Be\x00Bf\x00Bo\x00Br\x00Bs\x00Bu\x00CH\x00CO\x00Ca\x00Cc\x00Cd\x00Ce\x00Cf\x00Ch\x00Ci\x00Cl\x00Co\x00Cr\x00Cs\x00Cu\x00DD\x00DJ\x00DS\x00DZ\x00Da\x00Dc\x00De\x00Df\x00Di\x00Do\x00Ds\x00EN\x00ET\x00Ea\x00Ec\x00Ed\x00Ef\x00Eg\x00El\x00Em\x00Eo\x00Ep\x00Eq\x00Es\x00Et\x00Eu\x00Ex\x00Fc\x00Ff\x00Fi\x00Fo\x00Fs\x00GJ\x00GT\x00Ga\x00Gb\x00Gc\x00Gd\x00Gf\x00Gg\x00Go\x00Gr\x00Gs\x00Gt\x00HA\x00Ha\x00Hc\x00Hf\x00Hi\x00Ho\x00Hs\x00Hu\x00IE\x00IJ\x00IO\x00Ia\x00Ic\x00Id\x00If\x00Ig\x00Im\x00In\x00Io\x00Is\x00It\x00Iu\x00Jc\x00Jf\x00Jo\x00Js\x00Ju\x00KH\x00KJ\x00Ka\x00Kc\x00Kf\x00Ko\x00Ks\x00LJ\x00LT\x00La\x00Lc\x00Le\x00Lf\x00Ll\x00Lm\x00Lo\x00Ls\x00Lt\x00Ma\x00Mc\x00Me\x00Mf\x00Mi\x00Mo\x00Ms\x00Mu\x00NJ\x00Na\x00Nc\x00Ne\x00Nf\x00No\x00Ns\x00Nt\x00Nu\x00OE\x00Oa\x00Oc\x00Od\x00Of\x00Og\x00Om\x00Oo\x00Op\x00Or\x00Os\x00Ot\x00Ou\x00Ov\x00Pa\x00Pc\x00Pf\x00Ph\x00Pi\x00Pl\x00Po\x00Pr\x00Ps\x00QU\x00Qf\x00Qo\x00Qs\x00RB\x00RE\x00Ra\x00Rc\x00Re\x00Rf\x00Rh\x00Ri\x00Ro\x00Rr\x00Rs\x00Ru\x00SH\x00SO\x00Sa\x00Sc\x00Sf\x00Sh\x00Si\x00Sm\x00So\x00Sq\x00Ss\x00St\x00Su\x00TH\x00TR\x00TS\x00Ta\x00Tc\x00Tf\x00Th\x00Ti\x00To\x00Tr\x00Ts\x00Ua\x00Ub\x00Uc\x00Ud\x00Uf\x00Ug\x00Um\x00Un\x00Uo\x00Up\x00Ur\x00Us\x00Ut\x00Uu\x00VD\x00Vb\x00Vc\x00Vd\x00Ve\x00Vf\x00Vo\x00Vs\x00Vv\x00Wc\x00We\x00Wf\x00Wo\x00Ws\x00Xf\x00Xi\x00Xo\x00Xs\x00YA\x00YI\x00YU\x00Ya\x00Yc\x00Yf\x00Yo\x00Ys\x00Yu\x00ZH\x00Za\x00Zc\x00Zd\x00Ze\x00Zf\x00Zo\x00Zs\x00aa\x00ab\x00ac\x00ae\x00af\x00ag\x00al\x00am\x00an\x00ao\x00ap\x00ar\x00as\x00at\x00au\x00aw\x00bN\x00ba\x00bb\x00bc\x00bd\x00be\x00bf\x00bi\x00bk\x00bl\x00bn\x00bo\x00bp\x00br\x00bs\x00bu\x00ca\x00cc\x00cd\x00ce\x00cf\x00ch\x00ci\x00cl\x00co\x00cr\x00cs\x00ct\x00cu\x00cw\x00cy\x00dA\x00dH\x00da\x00db\x00dc\x00dd\x00de\x00df\x00dh\x00di\x00dj\x00dl\x00do\x00dr\x00ds\x00dt\x00du\x00dw\x00dz\x00eD\x00ea\x00ec\x00ed\x00ee\x00ef\x00eg\x00el\x00em\x00en\x00eo\x00ep\x00eq\x00er\x00es\x00et\x00eu\x00ex\x00fa\x00fc\x00fe\x00ff\x00fi\x00fj\x00fl\x00fn\x00fo\x00fp\x00fr\x00fs\x00gE\x00ga\x00gb\x00gc\x00gd\x00ge\x00gf\x00gg\x00gi\x00gj\x00gl\x00gn\x00go\x00gr\x00gs\x00gt\x00gv\x00hA\x00ha\x00hb\x00hc\x00he\x00hf\x00hk\x00ho\x00hs\x00hy\x00ia\x00ic\x00ie\x00if\x00ig\x00ii\x00ij\x00im\x00in\x00io\x00ip\x00iq\x00is\x00it\x00iu\x00jc\x00jf\x00jm\x00jo\x00js\x00ju\x00ka\x00kc\x00kf\x00kg\x00kh\x00kj\x00ko\x00ks\x00lA\x00lB\x00lE\x00lH\x00la\x00lb\x00lc\x00ld\x00le\x00lf\x00lg\x00lh\x00lj\x00ll\x00lm\x00ln\x00lo\x00lp\x00lr\x00ls\x00lt\x00lu\x00lv\x00mD\x00ma\x00mc\x00md\x00me\x00mf\x00mh\x00mi\x00ml\x00mn\x00mo\x00mp\x00ms\x00mu\x00nG\x00nL\x00nR\x00nV\x00na\x00nb\x00nc\x00nd\x00ne\x00nf\x00ng\x00nh\x00ni\x00nj\x00nl\x00nm\x00no\x00np\x00nr\x00ns\x00nt\x00nu\x00nv\x00nw\x00oS\x00oa\x00oc\x00od\x00oe\x00of\x00og\x00oh\x00oi\x00ol\x00om\x00oo\x00op\x00or\x00os\x00ot\x00ou\x00ov\x00pa\x00pc\x00pe\x00pf\x00ph\x00pi\x00pl\x00pm\x00po\x00pr\x00ps\x00pu\x00qf\x00qi\x00qo\x00qp\x00qs\x00qu\x00rA\x00rB\x00rH\x00ra\x00rb\x00rc\x00rd\x00re\x00rf\x00rh\x00ri\x00rl\x00rm\x00rn\x00ro\x00rp\x00rr\x00rs\x00rt\x00ru\x00rx\x00sa\x00sb\x00sc\x00sd\x00se\x00sf\x00sh\x00si\x00sl\x00sm\x00so\x00sp\x00sq\x00sr\x00ss\x00st\x00su\x00sw\x00sz\x00ta\x00tb\x00tc\x00td\x00te\x00tf\x00th\x00ti\x00to\x00tp\x00tr\x00ts\x00tw\x00uA\x00uH\x00ua\x00ub\x00uc\x00ud\x00uf\x00ug\x00uh\x00ul\x00um\x00uo\x00up\x00ur\x00us\x00ut\x00uu\x00uw\x00vA\x00vB\x00vD\x00va\x00vc\x00vd\x00ve\x00vf\x00vl\x00vn\x00vo\x00vp\x00vr\x00vs\x00vz\x00wc\x00we\x00wf\x00wo\x00wp\x00wr\x00ws\x00xc\x00xd\x00xf\x00xh\x00xi\x00xl\x00xm\x00xn\x00xo\x00xr\x00xs\x00xu\x00xv\x00xw\x00ya\x00yc\x00ye\x00yf\x00yi\x00yo\x00ys\x00yu\x00za\x00zc\x00zd\x00ze\x00zf\x00zh\x00zi\x00zo\x00zs\x00zw\x00",
		"large_words" => array(
			// AElig;[Æ] AElig[Æ].
			"\x04lig;\x02Æ\x03lig\x02Æ",
			// AMP;[&] AMP[&].
			"\x02P;\x01&\x01P\x01&",
			// Aacute;[Á] Aacute[Á].
			"\x05cute;\x02Á\x04cute\x02Á",
			// Abreve;[Ă].
			"\x05reve;\x02Ă",
			// Acirc;[Â] Acirc[Â] Acy;[А].
			"\x04irc;\x02Â\x03irc\x02Â\x02y;\x02А",
			// Afr;[𝔄].
			"\x02r;\x04𝔄",
			// Agrave;[À] Agrave[À].
			"\x05rave;\x02À\x04rave\x02À",
			// Alpha;[Α].
			"\x04pha;\x02Α",
			// Amacr;[Ā].
			"\x04acr;\x02Ā",
			// And;[⩓].
			"\x02d;\x03⩓",
			// Aogon;[Ą] Aopf;[𝔸].
			"\x04gon;\x02Ą\x03pf;\x04𝔸",
			// ApplyFunction;[⁡].
			"\x0cplyFunction;\x03⁡",
			// Aring;[Å] Aring[Å].
			"\x04ing;\x02Å\x03ing\x02Å",
			// Assign;[≔] Ascr;[𝒜].
			"\x05sign;\x03≔\x03cr;\x04𝒜",
			// Atilde;[Ã] Atilde[Ã].
			"\x05ilde;\x02Ã\x04ilde\x02Ã",
			// Auml;[Ä] Auml[Ä].
			"\x03ml;\x02Ä\x02ml\x02Ä",
			// Backslash;[∖] Barwed;[⌆] Barv;[⫧].
			"\x08ckslash;\x03∖\x05rwed;\x03⌆\x03rv;\x03⫧",
			// Bcy;[Б].
			"\x02y;\x02Б",
			// Bernoullis;[ℬ] Because;[∵] Beta;[Β].
			"\x09rnoullis;\x03ℬ\x06cause;\x03∵\x03ta;\x02Β",
			// Bfr;[𝔅].
			"\x02r;\x04𝔅",
			// Bopf;[𝔹].
			"\x03pf;\x04𝔹",
			// Breve;[˘].
			"\x04eve;\x02˘",
			// Bscr;[ℬ].
			"\x03cr;\x03ℬ",
			// Bumpeq;[≎].
			"\x05mpeq;\x03≎",
			// CHcy;[Ч].
			"\x03cy;\x02Ч",
			// COPY;[©] COPY[©].
			"\x03PY;\x02©\x02PY\x02©",
			// CapitalDifferentialD;[ⅅ] Cayleys;[ℭ] Cacute;[Ć] Cap;[⋒].
			"\x13pitalDifferentialD;\x03ⅅ\x06yleys;\x03ℭ\x05cute;\x02Ć\x02p;\x03⋒",
			// Cconint;[∰] Ccaron;[Č] Ccedil;[Ç] Ccedil[Ç] Ccirc;[Ĉ].
			"\x06onint;\x03∰\x05aron;\x02Č\x05edil;\x02Ç\x04edil\x02Ç\x04irc;\x02Ĉ",
			// Cdot;[Ċ].
			"\x03ot;\x02Ċ",
			// CenterDot;[·] Cedilla;[¸].
			"\x08nterDot;\x02·\x06dilla;\x02¸",
			// Cfr;[ℭ].
			"\x02r;\x03ℭ",
			// Chi;[Χ].
			"\x02i;\x02Χ",
			// CircleMinus;[⊖] CircleTimes;[⊗] CirclePlus;[⊕] CircleDot;[⊙].
			"\x0arcleMinus;\x03⊖\x0arcleTimes;\x03⊗\x09rclePlus;\x03⊕\x08rcleDot;\x03⊙",
			// ClockwiseContourIntegral;[∲] CloseCurlyDoubleQuote;[”] CloseCurlyQuote;[’].
			"\x17ockwiseContourIntegral;\x03∲\x14oseCurlyDoubleQuote;\x03”\x0eoseCurlyQuote;\x03’",
			// CounterClockwiseContourIntegral;[∳] ContourIntegral;[∮] Congruent;[≡] Coproduct;[∐] Colone;[⩴] Conint;[∯] Colon;[∷] Copf;[ℂ].
			"\x1eunterClockwiseContourIntegral;\x03∳\x0entourIntegral;\x03∮\x08ngruent;\x03≡\x08product;\x03∐\x05lone;\x03⩴\x05nint;\x03∯\x04lon;\x03∷\x03pf;\x03ℂ",
			// Cross;[⨯].
			"\x04oss;\x03⨯",
			// Cscr;[𝒞].
			"\x03cr;\x04𝒞",
			// CupCap;[≍] Cup;[⋓].
			"\x05pCap;\x03≍\x02p;\x03⋓",
			// DDotrahd;[⤑] DD;[ⅅ].
			"\x07otrahd;\x03⤑\x01;\x03ⅅ",
			// DJcy;[Ђ].
			"\x03cy;\x02Ђ",
			// DScy;[Ѕ].
			"\x03cy;\x02Ѕ",
			// DZcy;[Џ].
			"\x03cy;\x02Џ",
			// Dagger;[‡] Dashv;[⫤] Darr;[↡].
			"\x05gger;\x03‡\x04shv;\x03⫤\x03rr;\x03↡",
			// Dcaron;[Ď] Dcy;[Д].
			"\x05aron;\x02Ď\x02y;\x02Д",
			// Delta;[Δ] Del;[∇].
			"\x04lta;\x02Δ\x02l;\x03∇",
			// Dfr;[𝔇].
			"\x02r;\x04𝔇",
			// DiacriticalDoubleAcute;[˝] DiacriticalAcute;[´] DiacriticalGrave;[`] DiacriticalTilde;[˜] DiacriticalDot;[˙] DifferentialD;[ⅆ] Diamond;[⋄].
			"\x15acriticalDoubleAcute;\x02˝\x0facriticalAcute;\x02´\x0facriticalGrave;\x01`\x0facriticalTilde;\x02˜\x0dacriticalDot;\x02˙\x0cfferentialD;\x03ⅆ\x06amond;\x03⋄",
			// DoubleLongLeftRightArrow;[⟺] DoubleContourIntegral;[∯] DoubleLeftRightArrow;[⇔] DoubleLongRightArrow;[⟹] DoubleLongLeftArrow;[⟸] DownLeftRightVector;[⥐] DownRightTeeVector;[⥟] DownRightVectorBar;[⥗] DoubleUpDownArrow;[⇕] DoubleVerticalBar;[∥] DownLeftTeeVector;[⥞] DownLeftVectorBar;[⥖] DoubleRightArrow;[⇒] DownArrowUpArrow;[⇵] DoubleDownArrow;[⇓] DoubleLeftArrow;[⇐] DownRightVector;[⇁] DoubleRightTee;[⊨] DownLeftVector;[↽] DoubleLeftTee;[⫤] DoubleUpArrow;[⇑] DownArrowBar;[⤓] DownTeeArrow;[↧] DoubleDot;[¨] DownArrow;[↓] DownBreve;[̑] Downarrow;[⇓] DotEqual;[≐] DownTee;[⊤] DotDot;[⃜] Dopf;[𝔻] Dot;[¨].
			"\x17ubleLongLeftRightArrow;\x03⟺\x14ubleContourIntegral;\x03∯\x13ubleLeftRightArrow;\x03⇔\x13ubleLongRightArrow;\x03⟹\x12ubleLongLeftArrow;\x03⟸\x12wnLeftRightVector;\x03⥐\x11wnRightTeeVector;\x03⥟\x11wnRightVectorBar;\x03⥗\x10ubleUpDownArrow;\x03⇕\x10ubleVerticalBar;\x03∥\x10wnLeftTeeVector;\x03⥞\x10wnLeftVectorBar;\x03⥖\x0fubleRightArrow;\x03⇒\x0fwnArrowUpArrow;\x03⇵\x0eubleDownArrow;\x03⇓\x0eubleLeftArrow;\x03⇐\x0ewnRightVector;\x03⇁\x0dubleRightTee;\x03⊨\x0dwnLeftVector;\x03↽\x0cubleLeftTee;\x03⫤\x0cubleUpArrow;\x03⇑\x0bwnArrowBar;\x03⤓\x0bwnTeeArrow;\x03↧\x08ubleDot;\x02¨\x08wnArrow;\x03↓\x08wnBreve;\x02̑\x08wnarrow;\x03⇓\x07tEqual;\x03≐\x06wnTee;\x03⊤\x05tDot;\x03⃜\x03pf;\x04𝔻\x02t;\x02¨",
			// Dstrok;[Đ] Dscr;[𝒟].
			"\x05trok;\x02Đ\x03cr;\x04𝒟",
			// ENG;[Ŋ].
			"\x02G;\x02Ŋ",
			// ETH;[Ð] ETH[Ð].
			"\x02H;\x02Ð\x01H\x02Ð",
			// Eacute;[É] Eacute[É].
			"\x05cute;\x02É\x04cute\x02É",
			// Ecaron;[Ě] Ecirc;[Ê] Ecirc[Ê] Ecy;[Э].
			"\x05aron;\x02Ě\x04irc;\x02Ê\x03irc\x02Ê\x02y;\x02Э",
			// Edot;[Ė].
			"\x03ot;\x02Ė",
			// Efr;[𝔈].
			"\x02r;\x04𝔈",
			// Egrave;[È] Egrave[È].
			"\x05rave;\x02È\x04rave\x02È",
			// Element;[∈].
			"\x06ement;\x03∈",
			// EmptyVerySmallSquare;[▫] EmptySmallSquare;[◻] Emacr;[Ē].
			"\x13ptyVerySmallSquare;\x03▫\x0fptySmallSquare;\x03◻\x04acr;\x02Ē",
			// Eogon;[Ę] Eopf;[𝔼].
			"\x04gon;\x02Ę\x03pf;\x04𝔼",
			// Epsilon;[Ε].
			"\x06silon;\x02Ε",
			// Equilibrium;[⇌] EqualTilde;[≂] Equal;[⩵].
			"\x0auilibrium;\x03⇌\x09ualTilde;\x03≂\x04ual;\x03⩵",
			// Escr;[ℰ] Esim;[⩳].
			"\x03cr;\x03ℰ\x03im;\x03⩳",
			// Eta;[Η].
			"\x02a;\x02Η",
			// Euml;[Ë] Euml[Ë].
			"\x03ml;\x02Ë\x02ml\x02Ë",
			// ExponentialE;[ⅇ] Exists;[∃].
			"\x0bponentialE;\x03ⅇ\x05ists;\x03∃",
			// Fcy;[Ф].
			"\x02y;\x02Ф",
			// Ffr;[𝔉].
			"\x02r;\x04𝔉",
			// FilledVerySmallSquare;[▪] FilledSmallSquare;[◼].
			"\x14lledVerySmallSquare;\x03▪\x10lledSmallSquare;\x03◼",
			// Fouriertrf;[ℱ] ForAll;[∀] Fopf;[𝔽].
			"\x09uriertrf;\x03ℱ\x05rAll;\x03∀\x03pf;\x04𝔽",
			// Fscr;[ℱ].
			"\x03cr;\x03ℱ",
			// GJcy;[Ѓ].
			"\x03cy;\x02Ѓ",
			// GT;[>].
			"\x01;\x01>",
			// Gammad;[Ϝ] Gamma;[Γ].
			"\x05mmad;\x02Ϝ\x04mma;\x02Γ",
			// Gbreve;[Ğ].
			"\x05reve;\x02Ğ",
			// Gcedil;[Ģ] Gcirc;[Ĝ] Gcy;[Г].
			"\x05edil;\x02Ģ\x04irc;\x02Ĝ\x02y;\x02Г",
			// Gdot;[Ġ].
			"\x03ot;\x02Ġ",
			// Gfr;[𝔊].
			"\x02r;\x04𝔊",
			// Gg;[⋙].
			"\x01;\x03⋙",
			// Gopf;[𝔾].
			"\x03pf;\x04𝔾",
			// GreaterSlantEqual;[⩾] GreaterEqualLess;[⋛] GreaterFullEqual;[≧] GreaterGreater;[⪢] GreaterEqual;[≥] GreaterTilde;[≳] GreaterLess;[≷].
			"\x10eaterSlantEqual;\x03⩾\x0featerEqualLess;\x03⋛\x0featerFullEqual;\x03≧\x0deaterGreater;\x03⪢\x0beaterEqual;\x03≥\x0beaterTilde;\x03≳\x0aeaterLess;\x03≷",
			// Gscr;[𝒢].
			"\x03cr;\x04𝒢",
			// Gt;[≫].
			"\x01;\x03≫",
			// HARDcy;[Ъ].
			"\x05RDcy;\x02Ъ",
			// Hacek;[ˇ] Hat;[^].
			"\x04cek;\x02ˇ\x02t;\x01^",
			// Hcirc;[Ĥ].
			"\x04irc;\x02Ĥ",
			// Hfr;[ℌ].
			"\x02r;\x03ℌ",
			// HilbertSpace;[ℋ].
			"\x0blbertSpace;\x03ℋ",
			// HorizontalLine;[─] Hopf;[ℍ].
			"\x0drizontalLine;\x03─\x03pf;\x03ℍ",
			// Hstrok;[Ħ] Hscr;[ℋ].
			"\x05trok;\x02Ħ\x03cr;\x03ℋ",
			// HumpDownHump;[≎] HumpEqual;[≏].
			"\x0bmpDownHump;\x03≎\x08mpEqual;\x03≏",
			// IEcy;[Е].
			"\x03cy;\x02Е",
			// IJlig;[Ĳ].
			"\x04lig;\x02Ĳ",
			// IOcy;[Ё].
			"\x03cy;\x02Ё",
			// Iacute;[Í] Iacute[Í].
			"\x05cute;\x02Í\x04cute\x02Í",
			// Icirc;[Î] Icirc[Î] Icy;[И].
			"\x04irc;\x02Î\x03irc\x02Î\x02y;\x02И",
			// Idot;[İ].
			"\x03ot;\x02İ",
			// Ifr;[ℑ].
			"\x02r;\x03ℑ",
			// Igrave;[Ì] Igrave[Ì].
			"\x05rave;\x02Ì\x04rave\x02Ì",
			// ImaginaryI;[ⅈ] Implies;[⇒] Imacr;[Ī] Im;[ℑ].
			"\x09aginaryI;\x03ⅈ\x06plies;\x03⇒\x04acr;\x02Ī\x01;\x03ℑ",
			// InvisibleComma;[⁣] InvisibleTimes;[⁢] Intersection;[⋂] Integral;[∫] Int;[∬].
			"\x0dvisibleComma;\x03⁣\x0dvisibleTimes;\x03⁢\x0btersection;\x03⋂\x07tegral;\x03∫\x02t;\x03∬",
			// Iogon;[Į] Iopf;[𝕀] Iota;[Ι].
			"\x04gon;\x02Į\x03pf;\x04𝕀\x03ta;\x02Ι",
			// Iscr;[ℐ].
			"\x03cr;\x03ℐ",
			// Itilde;[Ĩ].
			"\x05ilde;\x02Ĩ",
			// Iukcy;[І] Iuml;[Ï] Iuml[Ï].
			"\x04kcy;\x02І\x03ml;\x02Ï\x02ml\x02Ï",
			// Jcirc;[Ĵ] Jcy;[Й].
			"\x04irc;\x02Ĵ\x02y;\x02Й",
			// Jfr;[𝔍].
			"\x02r;\x04𝔍",
			// Jopf;[𝕁].
			"\x03pf;\x04𝕁",
			// Jsercy;[Ј] Jscr;[𝒥].
			"\x05ercy;\x02Ј\x03cr;\x04𝒥",
			// Jukcy;[Є].
			"\x04kcy;\x02Є",
			// KHcy;[Х].
			"\x03cy;\x02Х",
			// KJcy;[Ќ].
			"\x03cy;\x02Ќ",
			// Kappa;[Κ].
			"\x04ppa;\x02Κ",
			// Kcedil;[Ķ] Kcy;[К].
			"\x05edil;\x02Ķ\x02y;\x02К",
			// Kfr;[𝔎].
			"\x02r;\x04𝔎",
			// Kopf;[𝕂].
			"\x03pf;\x04𝕂",
			// Kscr;[𝒦].
			"\x03cr;\x04𝒦",
			// LJcy;[Љ].
			"\x03cy;\x02Љ",
			// LT;[<].
			"\x01;\x01<",
			// Laplacetrf;[ℒ] Lacute;[Ĺ] Lambda;[Λ] Lang;[⟪] Larr;[↞].
			"\x09placetrf;\x03ℒ\x05cute;\x02Ĺ\x05mbda;\x02Λ\x03ng;\x03⟪\x03rr;\x03↞",
			// Lcaron;[Ľ] Lcedil;[Ļ] Lcy;[Л].
			"\x05aron;\x02Ľ\x05edil;\x02Ļ\x02y;\x02Л",
			// LeftArrowRightArrow;[⇆] LeftDoubleBracket;[⟦] LeftDownTeeVector;[⥡] LeftDownVectorBar;[⥙] LeftTriangleEqual;[⊴] LeftAngleBracket;[⟨] LeftUpDownVector;[⥑] LessEqualGreater;[⋚] LeftRightVector;[⥎] LeftTriangleBar;[⧏] LeftUpTeeVector;[⥠] LeftUpVectorBar;[⥘] LeftDownVector;[⇃] LeftRightArrow;[↔] Leftrightarrow;[⇔] LessSlantEqual;[⩽] LeftTeeVector;[⥚] LeftVectorBar;[⥒] LessFullEqual;[≦] LeftArrowBar;[⇤] LeftTeeArrow;[↤] LeftTriangle;[⊲] LeftUpVector;[↿] LeftCeiling;[⌈] LessGreater;[≶] LeftVector;[↼] LeftArrow;[←] LeftFloor;[⌊] Leftarrow;[⇐] LessTilde;[≲] LessLess;[⪡] LeftTee;[⊣].
			"\x12ftArrowRightArrow;\x03⇆\x10ftDoubleBracket;\x03⟦\x10ftDownTeeVector;\x03⥡\x10ftDownVectorBar;\x03⥙\x10ftTriangleEqual;\x03⊴\x0fftAngleBracket;\x03⟨\x0fftUpDownVector;\x03⥑\x0fssEqualGreater;\x03⋚\x0eftRightVector;\x03⥎\x0eftTriangleBar;\x03⧏\x0eftUpTeeVector;\x03⥠\x0eftUpVectorBar;\x03⥘\x0dftDownVector;\x03⇃\x0dftRightArrow;\x03↔\x0dftrightarrow;\x03⇔\x0dssSlantEqual;\x03⩽\x0cftTeeVector;\x03⥚\x0cftVectorBar;\x03⥒\x0cssFullEqual;\x03≦\x0bftArrowBar;\x03⇤\x0bftTeeArrow;\x03↤\x0bftTriangle;\x03⊲\x0bftUpVector;\x03↿\x0aftCeiling;\x03⌈\x0assGreater;\x03≶\x09ftVector;\x03↼\x08ftArrow;\x03←\x08ftFloor;\x03⌊\x08ftarrow;\x03⇐\x08ssTilde;\x03≲\x07ssLess;\x03⪡\x06ftTee;\x03⊣",
			// Lfr;[𝔏].
			"\x02r;\x04𝔏",
			// Lleftarrow;[⇚] Ll;[⋘].
			"\x09eftarrow;\x03⇚\x01;\x03⋘",
			// Lmidot;[Ŀ].
			"\x05idot;\x02Ŀ",
			// LongLeftRightArrow;[⟷] Longleftrightarrow;[⟺] LowerRightArrow;[↘] LongRightArrow;[⟶] Longrightarrow;[⟹] LowerLeftArrow;[↙] LongLeftArrow;[⟵] Longleftarrow;[⟸] Lopf;[𝕃].
			"\x11ngLeftRightArrow;\x03⟷\x11ngleftrightarrow;\x03⟺\x0ewerRightArrow;\x03↘\x0dngRightArrow;\x03⟶\x0dngrightarrow;\x03⟹\x0dwerLeftArrow;\x03↙\x0cngLeftArrow;\x03⟵\x0cngleftarrow;\x03⟸\x03pf;\x04𝕃",
			// Lstrok;[Ł] Lscr;[ℒ] Lsh;[↰].
			"\x05trok;\x02Ł\x03cr;\x03ℒ\x02h;\x03↰",
			// Lt;[≪].
			"\x01;\x03≪",
			// Map;[⤅].
			"\x02p;\x03⤅",
			// Mcy;[М].
			"\x02y;\x02М",
			// MediumSpace;[ ] Mellintrf;[ℳ].
			"\x0adiumSpace;\x03 \x08llintrf;\x03ℳ",
			// Mfr;[𝔐].
			"\x02r;\x04𝔐",
			// MinusPlus;[∓].
			"\x08nusPlus;\x03∓",
			// Mopf;[𝕄].
			"\x03pf;\x04𝕄",
			// Mscr;[ℳ].
			"\x03cr;\x03ℳ",
			// Mu;[Μ].
			"\x01;\x02Μ",
			// NJcy;[Њ].
			"\x03cy;\x02Њ",
			// Nacute;[Ń].
			"\x05cute;\x02Ń",
			// Ncaron;[Ň] Ncedil;[Ņ] Ncy;[Н].
			"\x05aron;\x02Ň\x05edil;\x02Ņ\x02y;\x02Н",
			// NegativeVeryThinSpace;[​] NestedGreaterGreater;[≫] NegativeMediumSpace;[​] NegativeThickSpace;[​] NegativeThinSpace;[​] NestedLessLess;[≪] NewLine;[\xa].
			"\x14gativeVeryThinSpace;\x03​\x13stedGreaterGreater;\x03≫\x12gativeMediumSpace;\x03​\x11gativeThickSpace;\x03​\x10gativeThinSpace;\x03​\x0dstedLessLess;\x03≪\x06wLine;\x01\xa",
			// Nfr;[𝔑].
			"\x02r;\x04𝔑",
			// NotNestedGreaterGreater;[⪢̸] NotSquareSupersetEqual;[⋣] NotPrecedesSlantEqual;[⋠] NotRightTriangleEqual;[⋭] NotSucceedsSlantEqual;[⋡] NotDoubleVerticalBar;[∦] NotGreaterSlantEqual;[⩾̸] NotLeftTriangleEqual;[⋬] NotSquareSubsetEqual;[⋢] NotGreaterFullEqual;[≧̸] NotRightTriangleBar;[⧐̸] NotLeftTriangleBar;[⧏̸] NotGreaterGreater;[≫̸] NotLessSlantEqual;[⩽̸] NotNestedLessLess;[⪡̸] NotReverseElement;[∌] NotSquareSuperset;[⊐̸] NotTildeFullEqual;[≇] NonBreakingSpace;[ ] NotPrecedesEqual;[⪯̸] NotRightTriangle;[⋫] NotSucceedsEqual;[⪰̸] NotSucceedsTilde;[≿̸] NotSupersetEqual;[⊉] NotGreaterEqual;[≱] NotGreaterTilde;[≵] NotHumpDownHump;[≎̸] NotLeftTriangle;[⋪] NotSquareSubset;[⊏̸] NotGreaterLess;[≹] NotLessGreater;[≸] NotSubsetEqual;[⊈] NotVerticalBar;[∤] NotEqualTilde;[≂̸] NotTildeEqual;[≄] NotTildeTilde;[≉] NotCongruent;[≢] NotHumpEqual;[≏̸] NotLessEqual;[≰] NotLessTilde;[≴] NotLessLess;[≪̸] NotPrecedes;[⊀] NotSucceeds;[⊁] NotSuperset;[⊃⃒] NotElement;[∉] NotGreater;[≯] NotCupCap;[≭] NotExists;[∄] NotSubset;[⊂⃒] NotEqual;[≠] NotTilde;[≁] NoBreak;[⁠] NotLess;[≮] Nopf;[ℕ] Not;[⫬].
			"\x16tNestedGreaterGreater;\x05⪢̸\x15tSquareSupersetEqual;\x03⋣\x14tPrecedesSlantEqual;\x03⋠\x14tRightTriangleEqual;\x03⋭\x14tSucceedsSlantEqual;\x03⋡\x13tDoubleVerticalBar;\x03∦\x13tGreaterSlantEqual;\x05⩾̸\x13tLeftTriangleEqual;\x03⋬\x13tSquareSubsetEqual;\x03⋢\x12tGreaterFullEqual;\x05≧̸\x12tRightTriangleBar;\x05⧐̸\x11tLeftTriangleBar;\x05⧏̸\x10tGreaterGreater;\x05≫̸\x10tLessSlantEqual;\x05⩽̸\x10tNestedLessLess;\x05⪡̸\x10tReverseElement;\x03∌\x10tSquareSuperset;\x05⊐̸\x10tTildeFullEqual;\x03≇\x0fnBreakingSpace;\x02 \x0ftPrecedesEqual;\x05⪯̸\x0ftRightTriangle;\x03⋫\x0ftSucceedsEqual;\x05⪰̸\x0ftSucceedsTilde;\x05≿̸\x0ftSupersetEqual;\x03⊉\x0etGreaterEqual;\x03≱\x0etGreaterTilde;\x03≵\x0etHumpDownHump;\x05≎̸\x0etLeftTriangle;\x03⋪\x0etSquareSubset;\x05⊏̸\x0dtGreaterLess;\x03≹\x0dtLessGreater;\x03≸\x0dtSubsetEqual;\x03⊈\x0dtVerticalBar;\x03∤\x0ctEqualTilde;\x05≂̸\x0ctTildeEqual;\x03≄\x0ctTildeTilde;\x03≉\x0btCongruent;\x03≢\x0btHumpEqual;\x05≏̸\x0btLessEqual;\x03≰\x0btLessTilde;\x03≴\x0atLessLess;\x05≪̸\x0atPrecedes;\x03⊀\x0atSucceeds;\x03⊁\x0atSuperset;\x06⊃⃒\x09tElement;\x03∉\x09tGreater;\x03≯\x08tCupCap;\x03≭\x08tExists;\x03∄\x08tSubset;\x06⊂⃒\x07tEqual;\x03≠\x07tTilde;\x03≁\x06Break;\x03⁠\x06tLess;\x03≮\x03pf;\x03ℕ\x02t;\x03⫬",
			// Nscr;[𝒩].
			"\x03cr;\x04𝒩",
			// Ntilde;[Ñ] Ntilde[Ñ].
			"\x05ilde;\x02Ñ\x04ilde\x02Ñ",
			// Nu;[Ν].
			"\x01;\x02Ν",
			// OElig;[Œ].
			"\x04lig;\x02Œ",
			// Oacute;[Ó] Oacute[Ó].
			"\x05cute;\x02Ó\x04cute\x02Ó",
			// Ocirc;[Ô] Ocirc[Ô] Ocy;[О].
			"\x04irc;\x02Ô\x03irc\x02Ô\x02y;\x02О",
			// Odblac;[Ő].
			"\x05blac;\x02Ő",
			// Ofr;[𝔒].
			"\x02r;\x04𝔒",
			// Ograve;[Ò] Ograve[Ò].
			"\x05rave;\x02Ò\x04rave\x02Ò",
			// Omicron;[Ο] Omacr;[Ō] Omega;[Ω].
			"\x06icron;\x02Ο\x04acr;\x02Ō\x04ega;\x02Ω",
			// Oopf;[𝕆].
			"\x03pf;\x04𝕆",
			// OpenCurlyDoubleQuote;[“] OpenCurlyQuote;[‘].
			"\x13enCurlyDoubleQuote;\x03“\x0denCurlyQuote;\x03‘",
			// Or;[⩔].
			"\x01;\x03⩔",
			// Oslash;[Ø] Oslash[Ø] Oscr;[𝒪].
			"\x05lash;\x02Ø\x04lash\x02Ø\x03cr;\x04𝒪",
			// Otilde;[Õ] Otimes;[⨷] Otilde[Õ].
			"\x05ilde;\x02Õ\x05imes;\x03⨷\x04ilde\x02Õ",
			// Ouml;[Ö] Ouml[Ö].
			"\x03ml;\x02Ö\x02ml\x02Ö",
			// OverParenthesis;[⏜] OverBracket;[⎴] OverBrace;[⏞] OverBar;[‾].
			"\x0eerParenthesis;\x03⏜\x0aerBracket;\x03⎴\x08erBrace;\x03⏞\x06erBar;\x03‾",
			// PartialD;[∂].
			"\x07rtialD;\x03∂",
			// Pcy;[П].
			"\x02y;\x02П",
			// Pfr;[𝔓].
			"\x02r;\x04𝔓",
			// Phi;[Φ].
			"\x02i;\x02Φ",
			// Pi;[Π].
			"\x01;\x02Π",
			// PlusMinus;[±].
			"\x08usMinus;\x02±",
			// Poincareplane;[ℌ] Popf;[ℙ].
			"\x0cincareplane;\x03ℌ\x03pf;\x03ℙ",
			// PrecedesSlantEqual;[≼] PrecedesEqual;[⪯] PrecedesTilde;[≾] Proportional;[∝] Proportion;[∷] Precedes;[≺] Product;[∏] Prime;[″] Pr;[⪻].
			"\x11ecedesSlantEqual;\x03≼\x0cecedesEqual;\x03⪯\x0cecedesTilde;\x03≾\x0boportional;\x03∝\x09oportion;\x03∷\x07ecedes;\x03≺\x06oduct;\x03∏\x04ime;\x03″\x01;\x03⪻",
			// Pscr;[𝒫] Psi;[Ψ].
			"\x03cr;\x04𝒫\x02i;\x02Ψ",
			// QUOT;[\"] QUOT[\"].
			"\x03OT;\x01\"\x02OT\x01\"",
			// Qfr;[𝔔].
			"\x02r;\x04𝔔",
			// Qopf;[ℚ].
			"\x03pf;\x03ℚ",
			// Qscr;[𝒬].
			"\x03cr;\x04𝒬",
			// RBarr;[⤐].
			"\x04arr;\x03⤐",
			// REG;[®] REG[®].
			"\x02G;\x02®\x01G\x02®",
			// Racute;[Ŕ] Rarrtl;[⤖] Rang;[⟫] Rarr;[↠].
			"\x05cute;\x02Ŕ\x05rrtl;\x03⤖\x03ng;\x03⟫\x03rr;\x03↠",
			// Rcaron;[Ř] Rcedil;[Ŗ] Rcy;[Р].
			"\x05aron;\x02Ř\x05edil;\x02Ŗ\x02y;\x02Р",
			// ReverseUpEquilibrium;[⥯] ReverseEquilibrium;[⇋] ReverseElement;[∋] Re;[ℜ].
			"\x13verseUpEquilibrium;\x03⥯\x11verseEquilibrium;\x03⇋\x0dverseElement;\x03∋\x01;\x03ℜ",
			// Rfr;[ℜ].
			"\x02r;\x03ℜ",
			// Rho;[Ρ].
			"\x02o;\x02Ρ",
			// RightArrowLeftArrow;[⇄] RightDoubleBracket;[⟧] RightDownTeeVector;[⥝] RightDownVectorBar;[⥕] RightTriangleEqual;[⊵] RightAngleBracket;[⟩] RightUpDownVector;[⥏] RightTriangleBar;[⧐] RightUpTeeVector;[⥜] RightUpVectorBar;[⥔] RightDownVector;[⇂] RightTeeVector;[⥛] RightVectorBar;[⥓] RightArrowBar;[⇥] RightTeeArrow;[↦] RightTriangle;[⊳] RightUpVector;[↾] RightCeiling;[⌉] RightVector;[⇀] RightArrow;[→] RightFloor;[⌋] Rightarrow;[⇒] RightTee;[⊢].
			"\x12ghtArrowLeftArrow;\x03⇄\x11ghtDoubleBracket;\x03⟧\x11ghtDownTeeVector;\x03⥝\x11ghtDownVectorBar;\x03⥕\x11ghtTriangleEqual;\x03⊵\x10ghtAngleBracket;\x03⟩\x10ghtUpDownVector;\x03⥏\x0fghtTriangleBar;\x03⧐\x0fghtUpTeeVector;\x03⥜\x0fghtUpVectorBar;\x03⥔\x0eghtDownVector;\x03⇂\x0dghtTeeVector;\x03⥛\x0dghtVectorBar;\x03⥓\x0cghtArrowBar;\x03⇥\x0cghtTeeArrow;\x03↦\x0cghtTriangle;\x03⊳\x0cghtUpVector;\x03↾\x0bghtCeiling;\x03⌉\x0aghtVector;\x03⇀\x09ghtArrow;\x03→\x09ghtFloor;\x03⌋\x09ghtarrow;\x03⇒\x07ghtTee;\x03⊢",
			// RoundImplies;[⥰] Ropf;[ℝ].
			"\x0bundImplies;\x03⥰\x03pf;\x03ℝ",
			// Rrightarrow;[⇛].
			"\x0aightarrow;\x03⇛",
			// Rscr;[ℛ] Rsh;[↱].
			"\x03cr;\x03ℛ\x02h;\x03↱",
			// RuleDelayed;[⧴].
			"\x0aleDelayed;\x03⧴",
			// SHCHcy;[Щ] SHcy;[Ш].
			"\x05CHcy;\x02Щ\x03cy;\x02Ш",
			// SOFTcy;[Ь].
			"\x05FTcy;\x02Ь",
			// Sacute;[Ś].
			"\x05cute;\x02Ś",
			// Scaron;[Š] Scedil;[Ş] Scirc;[Ŝ] Scy;[С] Sc;[⪼].
			"\x05aron;\x02Š\x05edil;\x02Ş\x04irc;\x02Ŝ\x02y;\x02С\x01;\x03⪼",
			// Sfr;[𝔖].
			"\x02r;\x04𝔖",
			// ShortRightArrow;[→] ShortDownArrow;[↓] ShortLeftArrow;[←] ShortUpArrow;[↑].
			"\x0eortRightArrow;\x03→\x0dortDownArrow;\x03↓\x0dortLeftArrow;\x03←\x0bortUpArrow;\x03↑",
			// Sigma;[Σ].
			"\x04gma;\x02Σ",
			// SmallCircle;[∘].
			"\x0aallCircle;\x03∘",
			// Sopf;[𝕊].
			"\x03pf;\x04𝕊",
			// SquareSupersetEqual;[⊒] SquareIntersection;[⊓] SquareSubsetEqual;[⊑] SquareSuperset;[⊐] SquareSubset;[⊏] SquareUnion;[⊔] Square;[□] Sqrt;[√].
			"\x12uareSupersetEqual;\x03⊒\x11uareIntersection;\x03⊓\x10uareSubsetEqual;\x03⊑\x0duareSuperset;\x03⊐\x0buareSubset;\x03⊏\x0auareUnion;\x03⊔\x05uare;\x03□\x03rt;\x03√",
			// Sscr;[𝒮].
			"\x03cr;\x04𝒮",
			// Star;[⋆].
			"\x03ar;\x03⋆",
			// SucceedsSlantEqual;[≽] SucceedsEqual;[⪰] SucceedsTilde;[≿] SupersetEqual;[⊇] SubsetEqual;[⊆] Succeeds;[≻] SuchThat;[∋] Superset;[⊃] Subset;[⋐] Supset;[⋑] Sub;[⋐] Sum;[∑] Sup;[⋑].
			"\x11cceedsSlantEqual;\x03≽\x0ccceedsEqual;\x03⪰\x0ccceedsTilde;\x03≿\x0cpersetEqual;\x03⊇\x0absetEqual;\x03⊆\x07cceeds;\x03≻\x07chThat;\x03∋\x07perset;\x03⊃\x05bset;\x03⋐\x05pset;\x03⋑\x02b;\x03⋐\x02m;\x03∑\x02p;\x03⋑",
			// THORN;[Þ] THORN[Þ].
			"\x04ORN;\x02Þ\x03ORN\x02Þ",
			// TRADE;[™].
			"\x04ADE;\x03™",
			// TSHcy;[Ћ] TScy;[Ц].
			"\x04Hcy;\x02Ћ\x03cy;\x02Ц",
			// Tab;[\x9] Tau;[Τ].
			"\x02b;\x01\x9\x02u;\x02Τ",
			// Tcaron;[Ť] Tcedil;[Ţ] Tcy;[Т].
			"\x05aron;\x02Ť\x05edil;\x02Ţ\x02y;\x02Т",
			// Tfr;[𝔗].
			"\x02r;\x04𝔗",
			// ThickSpace;[  ] Therefore;[∴] ThinSpace;[ ] Theta;[Θ].
			"\x09ickSpace;\x06  \x08erefore;\x03∴\x08inSpace;\x03 \x04eta;\x02Θ",
			// TildeFullEqual;[≅] TildeEqual;[≃] TildeTilde;[≈] Tilde;[∼].
			"\x0dldeFullEqual;\x03≅\x09ldeEqual;\x03≃\x09ldeTilde;\x03≈\x04lde;\x03∼",
			// Topf;[𝕋].
			"\x03pf;\x04𝕋",
			// TripleDot;[⃛].
			"\x08ipleDot;\x03⃛",
			// Tstrok;[Ŧ] Tscr;[𝒯].
			"\x05trok;\x02Ŧ\x03cr;\x04𝒯",
			// Uarrocir;[⥉] Uacute;[Ú] Uacute[Ú] Uarr;[↟].
			"\x07rrocir;\x03⥉\x05cute;\x02Ú\x04cute\x02Ú\x03rr;\x03↟",
			// Ubreve;[Ŭ] Ubrcy;[Ў].
			"\x05reve;\x02Ŭ\x04rcy;\x02Ў",
			// Ucirc;[Û] Ucirc[Û] Ucy;[У].
			"\x04irc;\x02Û\x03irc\x02Û\x02y;\x02У",
			// Udblac;[Ű].
			"\x05blac;\x02Ű",
			// Ufr;[𝔘].
			"\x02r;\x04𝔘",
			// Ugrave;[Ù] Ugrave[Ù].
			"\x05rave;\x02Ù\x04rave\x02Ù",
			// Umacr;[Ū].
			"\x04acr;\x02Ū",
			// UnderParenthesis;[⏝] UnderBracket;[⎵] UnderBrace;[⏟] UnionPlus;[⊎] UnderBar;[_] Union;[⋃].
			"\x0fderParenthesis;\x03⏝\x0bderBracket;\x03⎵\x09derBrace;\x03⏟\x08ionPlus;\x03⊎\x07derBar;\x01_\x04ion;\x03⋃",
			// Uogon;[Ų] Uopf;[𝕌].
			"\x04gon;\x02Ų\x03pf;\x04𝕌",
			// UpArrowDownArrow;[⇅] UpperRightArrow;[↗] UpperLeftArrow;[↖] UpEquilibrium;[⥮] UpDownArrow;[↕] Updownarrow;[⇕] UpArrowBar;[⤒] UpTeeArrow;[↥] UpArrow;[↑] Uparrow;[⇑] Upsilon;[Υ] UpTee;[⊥] Upsi;[ϒ].
			"\x0fArrowDownArrow;\x03⇅\x0eperRightArrow;\x03↗\x0dperLeftArrow;\x03↖\x0cEquilibrium;\x03⥮\x0aDownArrow;\x03↕\x0adownarrow;\x03⇕\x09ArrowBar;\x03⤒\x09TeeArrow;\x03↥\x06Arrow;\x03↑\x06arrow;\x03⇑\x06silon;\x02Υ\x04Tee;\x03⊥\x03si;\x02ϒ",
			// Uring;[Ů].
			"\x04ing;\x02Ů",
			// Uscr;[𝒰].
			"\x03cr;\x04𝒰",
			// Utilde;[Ũ].
			"\x05ilde;\x02Ũ",
			// Uuml;[Ü] Uuml[Ü].
			"\x03ml;\x02Ü\x02ml\x02Ü",
			// VDash;[⊫].
			"\x04ash;\x03⊫",
			// Vbar;[⫫].
			"\x03ar;\x03⫫",
			// Vcy;[В].
			"\x02y;\x02В",
			// Vdashl;[⫦] Vdash;[⊩].
			"\x05ashl;\x03⫦\x04ash;\x03⊩",
			// VerticalSeparator;[❘] VerticalTilde;[≀] VeryThinSpace;[ ] VerticalLine;[|] VerticalBar;[∣] Verbar;[‖] Vert;[‖] Vee;[⋁].
			"\x10rticalSeparator;\x03❘\x0crticalTilde;\x03≀\x0cryThinSpace;\x03 \x0brticalLine;\x01|\x0articalBar;\x03∣\x05rbar;\x03‖\x03rt;\x03‖\x02e;\x03⋁",
			// Vfr;[𝔙].
			"\x02r;\x04𝔙",
			// Vopf;[𝕍].
			"\x03pf;\x04𝕍",
			// Vscr;[𝒱].
			"\x03cr;\x04𝒱",
			// Vvdash;[⊪].
			"\x05dash;\x03⊪",
			// Wcirc;[Ŵ].
			"\x04irc;\x02Ŵ",
			// Wedge;[⋀].
			"\x04dge;\x03⋀",
			// Wfr;[𝔚].
			"\x02r;\x04𝔚",
			// Wopf;[𝕎].
			"\x03pf;\x04𝕎",
			// Wscr;[𝒲].
			"\x03cr;\x04𝒲",
			// Xfr;[𝔛].
			"\x02r;\x04𝔛",
			// Xi;[Ξ].
			"\x01;\x02Ξ",
			// Xopf;[𝕏].
			"\x03pf;\x04𝕏",
			// Xscr;[𝒳].
			"\x03cr;\x04𝒳",
			// YAcy;[Я].
			"\x03cy;\x02Я",
			// YIcy;[Ї].
			"\x03cy;\x02Ї",
			// YUcy;[Ю].
			"\x03cy;\x02Ю",
			// Yacute;[Ý] Yacute[Ý].
			"\x05cute;\x02Ý\x04cute\x02Ý",
			// Ycirc;[Ŷ] Ycy;[Ы].
			"\x04irc;\x02Ŷ\x02y;\x02Ы",
			// Yfr;[𝔜].
			"\x02r;\x04𝔜",
			// Yopf;[𝕐].
			"\x03pf;\x04𝕐",
			// Yscr;[𝒴].
			"\x03cr;\x04𝒴",
			// Yuml;[Ÿ].
			"\x03ml;\x02Ÿ",
			// ZHcy;[Ж].
			"\x03cy;\x02Ж",
			// Zacute;[Ź].
			"\x05cute;\x02Ź",
			// Zcaron;[Ž] Zcy;[З].
			"\x05aron;\x02Ž\x02y;\x02З",
			// Zdot;[Ż].
			"\x03ot;\x02Ż",
			// ZeroWidthSpace;[​] Zeta;[Ζ].
			"\x0droWidthSpace;\x03​\x03ta;\x02Ζ",
			// Zfr;[ℨ].
			"\x02r;\x03ℨ",
			// Zopf;[ℤ].
			"\x03pf;\x03ℤ",
			// Zscr;[𝒵].
			"\x03cr;\x04𝒵",
			// aacute;[á] aacute[á].
			"\x05cute;\x02á\x04cute\x02á",
			// abreve;[ă].
			"\x05reve;\x02ă",
			// acirc;[â] acute;[´] acirc[â] acute[´] acE;[∾̳] acd;[∿] acy;[а] ac;[∾].
			"\x04irc;\x02â\x04ute;\x02´\x03irc\x02â\x03ute\x02´\x02E;\x05∾̳\x02d;\x03∿\x02y;\x02а\x01;\x03∾",
			// aelig;[æ] aelig[æ].
			"\x04lig;\x02æ\x03lig\x02æ",
			// afr;[𝔞] af;[⁡].
			"\x02r;\x04𝔞\x01;\x03⁡",
			// agrave;[à] agrave[à].
			"\x05rave;\x02à\x04rave\x02à",
			// alefsym;[ℵ] aleph;[ℵ] alpha;[α].
			"\x06efsym;\x03ℵ\x04eph;\x03ℵ\x04pha;\x02α",
			// amacr;[ā] amalg;[⨿] amp;[&] amp[&].
			"\x04acr;\x02ā\x04alg;\x03⨿\x02p;\x01&\x01p\x01&",
			// andslope;[⩘] angmsdaa;[⦨] angmsdab;[⦩] angmsdac;[⦪] angmsdad;[⦫] angmsdae;[⦬] angmsdaf;[⦭] angmsdag;[⦮] angmsdah;[⦯] angrtvbd;[⦝] angrtvb;[⊾] angzarr;[⍼] andand;[⩕] angmsd;[∡] angsph;[∢] angle;[∠] angrt;[∟] angst;[Å] andd;[⩜] andv;[⩚] ange;[⦤] and;[∧] ang;[∠].
			"\x07dslope;\x03⩘\x07gmsdaa;\x03⦨\x07gmsdab;\x03⦩\x07gmsdac;\x03⦪\x07gmsdad;\x03⦫\x07gmsdae;\x03⦬\x07gmsdaf;\x03⦭\x07gmsdag;\x03⦮\x07gmsdah;\x03⦯\x07grtvbd;\x03⦝\x06grtvb;\x03⊾\x06gzarr;\x03⍼\x05dand;\x03⩕\x05gmsd;\x03∡\x05gsph;\x03∢\x04gle;\x03∠\x04grt;\x03∟\x04gst;\x02Å\x03dd;\x03⩜\x03dv;\x03⩚\x03ge;\x03⦤\x02d;\x03∧\x02g;\x03∠",
			// aogon;[ą] aopf;[𝕒].
			"\x04gon;\x02ą\x03pf;\x04𝕒",
			// approxeq;[≊] apacir;[⩯] approx;[≈] apid;[≋] apos;['] apE;[⩰] ape;[≊] ap;[≈].
			"\x07proxeq;\x03≊\x05acir;\x03⩯\x05prox;\x03≈\x03id;\x03≋\x03os;\x01'\x02E;\x03⩰\x02e;\x03≊\x01;\x03≈",
			// aring;[å] aring[å].
			"\x04ing;\x02å\x03ing\x02å",
			// asympeq;[≍] asymp;[≈] ascr;[𝒶] ast;[*].
			"\x06ympeq;\x03≍\x04ymp;\x03≈\x03cr;\x04𝒶\x02t;\x01*",
			// atilde;[ã] atilde[ã].
			"\x05ilde;\x02ã\x04ilde\x02ã",
			// auml;[ä] auml[ä].
			"\x03ml;\x02ä\x02ml\x02ä",
			// awconint;[∳] awint;[⨑].
			"\x07conint;\x03∳\x04int;\x03⨑",
			// bNot;[⫭].
			"\x03ot;\x03⫭",
			// backepsilon;[϶] backprime;[‵] backsimeq;[⋍] backcong;[≌] barwedge;[⌅] backsim;[∽] barvee;[⊽] barwed;[⌅].
			"\x0ackepsilon;\x02϶\x08ckprime;\x03‵\x08cksimeq;\x03⋍\x07ckcong;\x03≌\x07rwedge;\x03⌅\x06cksim;\x03∽\x05rvee;\x03⊽\x05rwed;\x03⌅",
			// bbrktbrk;[⎶] bbrk;[⎵].
			"\x07rktbrk;\x03⎶\x03rk;\x03⎵",
			// bcong;[≌] bcy;[б].
			"\x04ong;\x03≌\x02y;\x02б",
			// bdquo;[„].
			"\x04quo;\x03„",
			// because;[∵] bemptyv;[⦰] between;[≬] becaus;[∵] bernou;[ℬ] bepsi;[϶] beta;[β] beth;[ℶ].
			"\x06cause;\x03∵\x06mptyv;\x03⦰\x06tween;\x03≬\x05caus;\x03∵\x05rnou;\x03ℬ\x04psi;\x02϶\x03ta;\x02β\x03th;\x03ℶ",
			// bfr;[𝔟].
			"\x02r;\x04𝔟",
			// bigtriangledown;[▽] bigtriangleup;[△] bigotimes;[⨂] bigoplus;[⨁] bigsqcup;[⨆] biguplus;[⨄] bigwedge;[⋀] bigcirc;[◯] bigodot;[⨀] bigstar;[★] bigcap;[⋂] bigcup;[⋃] bigvee;[⋁].
			"\x0egtriangledown;\x03▽\x0cgtriangleup;\x03△\x08gotimes;\x03⨂\x07goplus;\x03⨁\x07gsqcup;\x03⨆\x07guplus;\x03⨄\x07gwedge;\x03⋀\x06gcirc;\x03◯\x06godot;\x03⨀\x06gstar;\x03★\x05gcap;\x03⋂\x05gcup;\x03⋃\x05gvee;\x03⋁",
			// bkarow;[⤍].
			"\x05arow;\x03⤍",
			// blacktriangleright;[▸] blacktriangledown;[▾] blacktriangleleft;[◂] blacktriangle;[▴] blacklozenge;[⧫] blacksquare;[▪] blank;[␣] blk12;[▒] blk14;[░] blk34;[▓] block;[█].
			"\x11acktriangleright;\x03▸\x10acktriangledown;\x03▾\x10acktriangleleft;\x03◂\x0cacktriangle;\x03▴\x0backlozenge;\x03⧫\x0aacksquare;\x03▪\x04ank;\x03␣\x04k12;\x03▒\x04k14;\x03░\x04k34;\x03▓\x04ock;\x03█",
			// bnequiv;[≡⃥] bnot;[⌐] bne;[=⃥].
			"\x06equiv;\x06≡⃥\x03ot;\x03⌐\x02e;\x04=⃥",
			// boxminus;[⊟] boxtimes;[⊠] boxplus;[⊞] bottom;[⊥] bowtie;[⋈] boxbox;[⧉] boxDL;[╗] boxDR;[╔] boxDl;[╖] boxDr;[╓] boxHD;[╦] boxHU;[╩] boxHd;[╤] boxHu;[╧] boxUL;[╝] boxUR;[╚] boxUl;[╜] boxUr;[╙] boxVH;[╬] boxVL;[╣] boxVR;[╠] boxVh;[╫] boxVl;[╢] boxVr;[╟] boxdL;[╕] boxdR;[╒] boxdl;[┐] boxdr;[┌] boxhD;[╥] boxhU;[╨] boxhd;[┬] boxhu;[┴] boxuL;[╛] boxuR;[╘] boxul;[┘] boxur;[└] boxvH;[╪] boxvL;[╡] boxvR;[╞] boxvh;[┼] boxvl;[┤] boxvr;[├] bopf;[𝕓] boxH;[═] boxV;[║] boxh;[─] boxv;[│] bot;[⊥].
			"\x07xminus;\x03⊟\x07xtimes;\x03⊠\x06xplus;\x03⊞\x05ttom;\x03⊥\x05wtie;\x03⋈\x05xbox;\x03⧉\x04xDL;\x03╗\x04xDR;\x03╔\x04xDl;\x03╖\x04xDr;\x03╓\x04xHD;\x03╦\x04xHU;\x03╩\x04xHd;\x03╤\x04xHu;\x03╧\x04xUL;\x03╝\x04xUR;\x03╚\x04xUl;\x03╜\x04xUr;\x03╙\x04xVH;\x03╬\x04xVL;\x03╣\x04xVR;\x03╠\x04xVh;\x03╫\x04xVl;\x03╢\x04xVr;\x03╟\x04xdL;\x03╕\x04xdR;\x03╒\x04xdl;\x03┐\x04xdr;\x03┌\x04xhD;\x03╥\x04xhU;\x03╨\x04xhd;\x03┬\x04xhu;\x03┴\x04xuL;\x03╛\x04xuR;\x03╘\x04xul;\x03┘\x04xur;\x03└\x04xvH;\x03╪\x04xvL;\x03╡\x04xvR;\x03╞\x04xvh;\x03┼\x04xvl;\x03┤\x04xvr;\x03├\x03pf;\x04𝕓\x03xH;\x03═\x03xV;\x03║\x03xh;\x03─\x03xv;\x03│\x02t;\x03⊥",
			// bprime;[‵].
			"\x05rime;\x03‵",
			// brvbar;[¦] breve;[˘] brvbar[¦].
			"\x05vbar;\x02¦\x04eve;\x02˘\x04vbar\x02¦",
			// bsolhsub;[⟈] bsemi;[⁏] bsime;[⋍] bsolb;[⧅] bscr;[𝒷] bsim;[∽] bsol;[\\].
			"\x07olhsub;\x03⟈\x04emi;\x03⁏\x04ime;\x03⋍\x04olb;\x03⧅\x03cr;\x04𝒷\x03im;\x03∽\x03ol;\x01\\",
			// bullet;[•] bumpeq;[≏] bumpE;[⪮] bumpe;[≏] bull;[•] bump;[≎].
			"\x05llet;\x03•\x05mpeq;\x03≏\x04mpE;\x03⪮\x04mpe;\x03≏\x03ll;\x03•\x03mp;\x03≎",
			// capbrcup;[⩉] cacute;[ć] capand;[⩄] capcap;[⩋] capcup;[⩇] capdot;[⩀] caret;[⁁] caron;[ˇ] caps;[∩︀] cap;[∩].
			"\x07pbrcup;\x03⩉\x05cute;\x02ć\x05pand;\x03⩄\x05pcap;\x03⩋\x05pcup;\x03⩇\x05pdot;\x03⩀\x04ret;\x03⁁\x04ron;\x02ˇ\x03ps;\x06∩︀\x02p;\x03∩",
			// ccupssm;[⩐] ccaron;[č] ccedil;[ç] ccaps;[⩍] ccedil[ç] ccirc;[ĉ] ccups;[⩌].
			"\x06upssm;\x03⩐\x05aron;\x02č\x05edil;\x02ç\x04aps;\x03⩍\x04edil\x02ç\x04irc;\x02ĉ\x04ups;\x03⩌",
			// cdot;[ċ].
			"\x03ot;\x02ċ",
			// centerdot;[·] cemptyv;[⦲] cedil;[¸] cedil[¸] cent;[¢] cent[¢].
			"\x08nterdot;\x02·\x06mptyv;\x03⦲\x04dil;\x02¸\x03dil\x02¸\x03nt;\x02¢\x02nt\x02¢",
			// cfr;[𝔠].
			"\x02r;\x04𝔠",
			// checkmark;[✓] check;[✓] chcy;[ч] chi;[χ].
			"\x08eckmark;\x03✓\x04eck;\x03✓\x03cy;\x02ч\x02i;\x02χ",
			// circlearrowright;[↻] circlearrowleft;[↺] circledcirc;[⊚] circleddash;[⊝] circledast;[⊛] circledR;[®] circledS;[Ⓢ] cirfnint;[⨐] cirscir;[⧂] circeq;[≗] cirmid;[⫯] cirE;[⧃] circ;[ˆ] cire;[≗] cir;[○].
			"\x0frclearrowright;\x03↻\x0erclearrowleft;\x03↺\x0arcledcirc;\x03⊚\x0arcleddash;\x03⊝\x09rcledast;\x03⊛\x07rcledR;\x02®\x07rcledS;\x03Ⓢ\x07rfnint;\x03⨐\x06rscir;\x03⧂\x05rceq;\x03≗\x05rmid;\x03⫯\x03rE;\x03⧃\x03rc;\x02ˆ\x03re;\x03≗\x02r;\x03○",
			// clubsuit;[♣] clubs;[♣].
			"\x07ubsuit;\x03♣\x04ubs;\x03♣",
			// complement;[∁] complexes;[ℂ] coloneq;[≔] congdot;[⩭] colone;[≔] commat;[@] compfn;[∘] conint;[∮] coprod;[∐] copysr;[℗] colon;[:] comma;[,] comp;[∁] cong;[≅] copf;[𝕔] copy;[©] copy[©].
			"\x09mplement;\x03∁\x08mplexes;\x03ℂ\x06loneq;\x03≔\x06ngdot;\x03⩭\x05lone;\x03≔\x05mmat;\x01@\x05mpfn;\x03∘\x05nint;\x03∮\x05prod;\x03∐\x05pysr;\x03℗\x04lon;\x01:\x04mma;\x01,\x03mp;\x03∁\x03ng;\x03≅\x03pf;\x04𝕔\x03py;\x02©\x02py\x02©",
			// crarr;[↵] cross;[✗].
			"\x04arr;\x03↵\x04oss;\x03✗",
			// csube;[⫑] csupe;[⫒] cscr;[𝒸] csub;[⫏] csup;[⫐].
			"\x04ube;\x03⫑\x04upe;\x03⫒\x03cr;\x04𝒸\x03ub;\x03⫏\x03up;\x03⫐",
			// ctdot;[⋯].
			"\x04dot;\x03⋯",
			// curvearrowright;[↷] curvearrowleft;[↶] curlyeqprec;[⋞] curlyeqsucc;[⋟] curlywedge;[⋏] cupbrcap;[⩈] curlyvee;[⋎] cudarrl;[⤸] cudarrr;[⤵] cularrp;[⤽] curarrm;[⤼] cularr;[↶] cupcap;[⩆] cupcup;[⩊] cupdot;[⊍] curarr;[↷] curren;[¤] cuepr;[⋞] cuesc;[⋟] cupor;[⩅] curren[¤] cuvee;[⋎] cuwed;[⋏] cups;[∪︀] cup;[∪].
			"\x0ervearrowright;\x03↷\x0drvearrowleft;\x03↶\x0arlyeqprec;\x03⋞\x0arlyeqsucc;\x03⋟\x09rlywedge;\x03⋏\x07pbrcap;\x03⩈\x07rlyvee;\x03⋎\x06darrl;\x03⤸\x06darrr;\x03⤵\x06larrp;\x03⤽\x06rarrm;\x03⤼\x05larr;\x03↶\x05pcap;\x03⩆\x05pcup;\x03⩊\x05pdot;\x03⊍\x05rarr;\x03↷\x05rren;\x02¤\x04epr;\x03⋞\x04esc;\x03⋟\x04por;\x03⩅\x04rren\x02¤\x04vee;\x03⋎\x04wed;\x03⋏\x03ps;\x06∪︀\x02p;\x03∪",
			// cwconint;[∲] cwint;[∱].
			"\x07conint;\x03∲\x04int;\x03∱",
			// cylcty;[⌭].
			"\x05lcty;\x03⌭",
			// dArr;[⇓].
			"\x03rr;\x03⇓",
			// dHar;[⥥].
			"\x03ar;\x03⥥",
			// dagger;[†] daleth;[ℸ] dashv;[⊣] darr;[↓] dash;[‐].
			"\x05gger;\x03†\x05leth;\x03ℸ\x04shv;\x03⊣\x03rr;\x03↓\x03sh;\x03‐",
			// dbkarow;[⤏] dblac;[˝].
			"\x06karow;\x03⤏\x04lac;\x02˝",
			// dcaron;[ď] dcy;[д].
			"\x05aron;\x02ď\x02y;\x02д",
			// ddagger;[‡] ddotseq;[⩷] ddarr;[⇊] dd;[ⅆ].
			"\x06agger;\x03‡\x06otseq;\x03⩷\x04arr;\x03⇊\x01;\x03ⅆ",
			// demptyv;[⦱] delta;[δ] deg;[°] deg[°].
			"\x06mptyv;\x03⦱\x04lta;\x02δ\x02g;\x02°\x01g\x02°",
			// dfisht;[⥿] dfr;[𝔡].
			"\x05isht;\x03⥿\x02r;\x04𝔡",
			// dharl;[⇃] dharr;[⇂].
			"\x04arl;\x03⇃\x04arr;\x03⇂",
			// divideontimes;[⋇] diamondsuit;[♦] diamond;[⋄] digamma;[ϝ] divide;[÷] divonx;[⋇] diams;[♦] disin;[⋲] divide[÷] diam;[⋄] die;[¨] div;[÷].
			"\x0cvideontimes;\x03⋇\x0aamondsuit;\x03♦\x06amond;\x03⋄\x06gamma;\x02ϝ\x05vide;\x02÷\x05vonx;\x03⋇\x04ams;\x03♦\x04sin;\x03⋲\x04vide\x02÷\x03am;\x03⋄\x02e;\x02¨\x02v;\x02÷",
			// djcy;[ђ].
			"\x03cy;\x02ђ",
			// dlcorn;[⌞] dlcrop;[⌍].
			"\x05corn;\x03⌞\x05crop;\x03⌍",
			// downharpoonright;[⇂] downharpoonleft;[⇃] doublebarwedge;[⌆] downdownarrows;[⇊] dotsquare;[⊡] downarrow;[↓] doteqdot;[≑] dotminus;[∸] dotplus;[∔] dollar;[$] doteq;[≐] dopf;[𝕕] dot;[˙].
			"\x0fwnharpoonright;\x03⇂\x0ewnharpoonleft;\x03⇃\x0dublebarwedge;\x03⌆\x0dwndownarrows;\x03⇊\x08tsquare;\x03⊡\x08wnarrow;\x03↓\x07teqdot;\x03≑\x07tminus;\x03∸\x06tplus;\x03∔\x05llar;\x01$\x04teq;\x03≐\x03pf;\x04𝕕\x02t;\x02˙",
			// drbkarow;[⤐] drcorn;[⌟] drcrop;[⌌].
			"\x07bkarow;\x03⤐\x05corn;\x03⌟\x05crop;\x03⌌",
			// dstrok;[đ] dscr;[𝒹] dscy;[ѕ] dsol;[⧶].
			"\x05trok;\x02đ\x03cr;\x04𝒹\x03cy;\x02ѕ\x03ol;\x03⧶",
			// dtdot;[⋱] dtrif;[▾] dtri;[▿].
			"\x04dot;\x03⋱\x04rif;\x03▾\x03ri;\x03▿",
			// duarr;[⇵] duhar;[⥯].
			"\x04arr;\x03⇵\x04har;\x03⥯",
			// dwangle;[⦦].
			"\x06angle;\x03⦦",
			// dzigrarr;[⟿] dzcy;[џ].
			"\x07igrarr;\x03⟿\x03cy;\x02џ",
			// eDDot;[⩷] eDot;[≑].
			"\x04Dot;\x03⩷\x03ot;\x03≑",
			// eacute;[é] easter;[⩮] eacute[é].
			"\x05cute;\x02é\x05ster;\x03⩮\x04cute\x02é",
			// ecaron;[ě] ecolon;[≕] ecirc;[ê] ecir;[≖] ecirc[ê] ecy;[э].
			"\x05aron;\x02ě\x05olon;\x03≕\x04irc;\x02ê\x03ir;\x03≖\x03irc\x02ê\x02y;\x02э",
			// edot;[ė].
			"\x03ot;\x02ė",
			// ee;[ⅇ].
			"\x01;\x03ⅇ",
			// efDot;[≒] efr;[𝔢].
			"\x04Dot;\x03≒\x02r;\x04𝔢",
			// egrave;[è] egsdot;[⪘] egrave[è] egs;[⪖] eg;[⪚].
			"\x05rave;\x02è\x05sdot;\x03⪘\x04rave\x02è\x02s;\x03⪖\x01;\x03⪚",
			// elinters;[⏧] elsdot;[⪗] ell;[ℓ] els;[⪕] el;[⪙].
			"\x07inters;\x03⏧\x05sdot;\x03⪗\x02l;\x03ℓ\x02s;\x03⪕\x01;\x03⪙",
			// emptyset;[∅] emptyv;[∅] emsp13;[ ] emsp14;[ ] emacr;[ē] empty;[∅] emsp;[ ].
			"\x07ptyset;\x03∅\x05ptyv;\x03∅\x05sp13;\x03 \x05sp14;\x03 \x04acr;\x02ē\x04pty;\x03∅\x03sp;\x03 ",
			// ensp;[ ] eng;[ŋ].
			"\x03sp;\x03 \x02g;\x02ŋ",
			// eogon;[ę] eopf;[𝕖].
			"\x04gon;\x02ę\x03pf;\x04𝕖",
			// epsilon;[ε] eparsl;[⧣] eplus;[⩱] epsiv;[ϵ] epar;[⋕] epsi;[ε].
			"\x06silon;\x02ε\x05arsl;\x03⧣\x04lus;\x03⩱\x04siv;\x02ϵ\x03ar;\x03⋕\x03si;\x02ε",
			// eqslantless;[⪕] eqslantgtr;[⪖] eqvparsl;[⧥] eqcolon;[≕] equivDD;[⩸] eqcirc;[≖] equals;[=] equest;[≟] eqsim;[≂] equiv;[≡].
			"\x0aslantless;\x03⪕\x09slantgtr;\x03⪖\x07vparsl;\x03⧥\x06colon;\x03≕\x06uivDD;\x03⩸\x05circ;\x03≖\x05uals;\x01=\x05uest;\x03≟\x04sim;\x03≂\x04uiv;\x03≡",
			// erDot;[≓] erarr;[⥱].
			"\x04Dot;\x03≓\x04arr;\x03⥱",
			// esdot;[≐] escr;[ℯ] esim;[≂].
			"\x04dot;\x03≐\x03cr;\x03ℯ\x03im;\x03≂",
			// eta;[η] eth;[ð] eth[ð].
			"\x02a;\x02η\x02h;\x02ð\x01h\x02ð",
			// euml;[ë] euro;[€] euml[ë].
			"\x03ml;\x02ë\x03ro;\x03€\x02ml\x02ë",
			// exponentiale;[ⅇ] expectation;[ℰ] exist;[∃] excl;[!].
			"\x0bponentiale;\x03ⅇ\x0apectation;\x03ℰ\x04ist;\x03∃\x03cl;\x01!",
			// fallingdotseq;[≒].
			"\x0cllingdotseq;\x03≒",
			// fcy;[ф].
			"\x02y;\x02ф",
			// female;[♀].
			"\x05male;\x03♀",
			// ffilig;[ﬃ] ffllig;[ﬄ] fflig;[ﬀ] ffr;[𝔣].
			"\x05ilig;\x03ﬃ\x05llig;\x03ﬄ\x04lig;\x03ﬀ\x02r;\x04𝔣",
			// filig;[ﬁ].
			"\x04lig;\x03ﬁ",
			// fjlig;[fj].
			"\x04lig;\x02fj",
			// fllig;[ﬂ] fltns;[▱] flat;[♭].
			"\x04lig;\x03ﬂ\x04tns;\x03▱\x03at;\x03♭",
			// fnof;[ƒ].
			"\x03of;\x02ƒ",
			// forall;[∀] forkv;[⫙] fopf;[𝕗] fork;[⋔].
			"\x05rall;\x03∀\x04rkv;\x03⫙\x03pf;\x04𝕗\x03rk;\x03⋔",
			// fpartint;[⨍].
			"\x07artint;\x03⨍",
			// frac12;[½] frac13;[⅓] frac14;[¼] frac15;[⅕] frac16;[⅙] frac18;[⅛] frac23;[⅔] frac25;[⅖] frac34;[¾] frac35;[⅗] frac38;[⅜] frac45;[⅘] frac56;[⅚] frac58;[⅝] frac78;[⅞] frac12[½] frac14[¼] frac34[¾] frasl;[⁄] frown;[⌢].
			"\x05ac12;\x02½\x05ac13;\x03⅓\x05ac14;\x02¼\x05ac15;\x03⅕\x05ac16;\x03⅙\x05ac18;\x03⅛\x05ac23;\x03⅔\x05ac25;\x03⅖\x05ac34;\x02¾\x05ac35;\x03⅗\x05ac38;\x03⅜\x05ac45;\x03⅘\x05ac56;\x03⅚\x05ac58;\x03⅝\x05ac78;\x03⅞\x04ac12\x02½\x04ac14\x02¼\x04ac34\x02¾\x04asl;\x03⁄\x04own;\x03⌢",
			// fscr;[𝒻].
			"\x03cr;\x04𝒻",
			// gEl;[⪌] gE;[≧].
			"\x02l;\x03⪌\x01;\x03≧",
			// gacute;[ǵ] gammad;[ϝ] gamma;[γ] gap;[⪆].
			"\x05cute;\x02ǵ\x05mmad;\x02ϝ\x04mma;\x02γ\x02p;\x03⪆",
			// gbreve;[ğ].
			"\x05reve;\x02ğ",
			// gcirc;[ĝ] gcy;[г].
			"\x04irc;\x02ĝ\x02y;\x02г",
			// gdot;[ġ].
			"\x03ot;\x02ġ",
			// geqslant;[⩾] gesdotol;[⪄] gesdoto;[⪂] gesdot;[⪀] gesles;[⪔] gescc;[⪩] geqq;[≧] gesl;[⋛︀] gel;[⋛] geq;[≥] ges;[⩾] ge;[≥].
			"\x07qslant;\x03⩾\x07sdotol;\x03⪄\x06sdoto;\x03⪂\x05sdot;\x03⪀\x05sles;\x03⪔\x04scc;\x03⪩\x03qq;\x03≧\x03sl;\x06⋛︀\x02l;\x03⋛\x02q;\x03≥\x02s;\x03⩾\x01;\x03≥",
			// gfr;[𝔤].
			"\x02r;\x04𝔤",
			// ggg;[⋙] gg;[≫].
			"\x02g;\x03⋙\x01;\x03≫",
			// gimel;[ℷ].
			"\x04mel;\x03ℷ",
			// gjcy;[ѓ].
			"\x03cy;\x02ѓ",
			// glE;[⪒] gla;[⪥] glj;[⪤] gl;[≷].
			"\x02E;\x03⪒\x02a;\x03⪥\x02j;\x03⪤\x01;\x03≷",
			// gnapprox;[⪊] gneqq;[≩] gnsim;[⋧] gnap;[⪊] gneq;[⪈] gnE;[≩] gne;[⪈].
			"\x07approx;\x03⪊\x04eqq;\x03≩\x04sim;\x03⋧\x03ap;\x03⪊\x03eq;\x03⪈\x02E;\x03≩\x02e;\x03⪈",
			// gopf;[𝕘].
			"\x03pf;\x04𝕘",
			// grave;[`].
			"\x04ave;\x01`",
			// gsime;[⪎] gsiml;[⪐] gscr;[ℊ] gsim;[≳].
			"\x04ime;\x03⪎\x04iml;\x03⪐\x03cr;\x03ℊ\x03im;\x03≳",
			// gtreqqless;[⪌] gtrapprox;[⪆] gtreqless;[⋛] gtquest;[⩼] gtrless;[≷] gtlPar;[⦕] gtrarr;[⥸] gtrdot;[⋗] gtrsim;[≳] gtcir;[⩺] gtdot;[⋗] gtcc;[⪧] gt;[>].
			"\x09reqqless;\x03⪌\x08rapprox;\x03⪆\x08reqless;\x03⋛\x06quest;\x03⩼\x06rless;\x03≷\x05lPar;\x03⦕\x05rarr;\x03⥸\x05rdot;\x03⋗\x05rsim;\x03≳\x04cir;\x03⩺\x04dot;\x03⋗\x03cc;\x03⪧\x01;\x01>",
			// gvertneqq;[≩︀] gvnE;[≩︀].
			"\x08ertneqq;\x06≩︀\x03nE;\x06≩︀",
			// hArr;[⇔].
			"\x03rr;\x03⇔",
			// harrcir;[⥈] hairsp;[ ] hamilt;[ℋ] hardcy;[ъ] harrw;[↭] half;[½] harr;[↔].
			"\x06rrcir;\x03⥈\x05irsp;\x03 \x05milt;\x03ℋ\x05rdcy;\x02ъ\x04rrw;\x03↭\x03lf;\x02½\x03rr;\x03↔",
			// hbar;[ℏ].
			"\x03ar;\x03ℏ",
			// hcirc;[ĥ].
			"\x04irc;\x02ĥ",
			// heartsuit;[♥] hearts;[♥] hellip;[…] hercon;[⊹].
			"\x08artsuit;\x03♥\x05arts;\x03♥\x05llip;\x03…\x05rcon;\x03⊹",
			// hfr;[𝔥].
			"\x02r;\x04𝔥",
			// hksearow;[⤥] hkswarow;[⤦].
			"\x07searow;\x03⤥\x07swarow;\x03⤦",
			// hookrightarrow;[↪] hookleftarrow;[↩] homtht;[∻] horbar;[―] hoarr;[⇿] hopf;[𝕙].
			"\x0dokrightarrow;\x03↪\x0cokleftarrow;\x03↩\x05mtht;\x03∻\x05rbar;\x03―\x04arr;\x03⇿\x03pf;\x04𝕙",
			// hslash;[ℏ] hstrok;[ħ] hscr;[𝒽].
			"\x05lash;\x03ℏ\x05trok;\x02ħ\x03cr;\x04𝒽",
			// hybull;[⁃] hyphen;[‐].
			"\x05bull;\x03⁃\x05phen;\x03‐",
			// iacute;[í] iacute[í].
			"\x05cute;\x02í\x04cute\x02í",
			// icirc;[î] icirc[î] icy;[и] ic;[⁣].
			"\x04irc;\x02î\x03irc\x02î\x02y;\x02и\x01;\x03⁣",
			// iexcl;[¡] iecy;[е] iexcl[¡].
			"\x04xcl;\x02¡\x03cy;\x02е\x03xcl\x02¡",
			// iff;[⇔] ifr;[𝔦].
			"\x02f;\x03⇔\x02r;\x04𝔦",
			// igrave;[ì] igrave[ì].
			"\x05rave;\x02ì\x04rave\x02ì",
			// iiiint;[⨌] iinfin;[⧜] iiint;[∭] iiota;[℩] ii;[ⅈ].
			"\x05iint;\x03⨌\x05nfin;\x03⧜\x04int;\x03∭\x04ota;\x03℩\x01;\x03ⅈ",
			// ijlig;[ĳ].
			"\x04lig;\x02ĳ",
			// imagline;[ℐ] imagpart;[ℑ] imacr;[ī] image;[ℑ] imath;[ı] imped;[Ƶ] imof;[⊷].
			"\x07agline;\x03ℐ\x07agpart;\x03ℑ\x04acr;\x02ī\x04age;\x03ℑ\x04ath;\x02ı\x04ped;\x02Ƶ\x03of;\x03⊷",
			// infintie;[⧝] integers;[ℤ] intercal;[⊺] intlarhk;[⨗] intprod;[⨼] incare;[℅] inodot;[ı] intcal;[⊺] infin;[∞] int;[∫] in;[∈].
			"\x07fintie;\x03⧝\x07tegers;\x03ℤ\x07tercal;\x03⊺\x07tlarhk;\x03⨗\x06tprod;\x03⨼\x05care;\x03℅\x05odot;\x02ı\x05tcal;\x03⊺\x04fin;\x03∞\x02t;\x03∫\x01;\x03∈",
			// iogon;[į] iocy;[ё] iopf;[𝕚] iota;[ι].
			"\x04gon;\x02į\x03cy;\x02ё\x03pf;\x04𝕚\x03ta;\x02ι",
			// iprod;[⨼].
			"\x04rod;\x03⨼",
			// iquest;[¿] iquest[¿].
			"\x05uest;\x02¿\x04uest\x02¿",
			// isindot;[⋵] isinsv;[⋳] isinE;[⋹] isins;[⋴] isinv;[∈] iscr;[𝒾] isin;[∈].
			"\x06indot;\x03⋵\x05insv;\x03⋳\x04inE;\x03⋹\x04ins;\x03⋴\x04inv;\x03∈\x03cr;\x04𝒾\x03in;\x03∈",
			// itilde;[ĩ] it;[⁢].
			"\x05ilde;\x02ĩ\x01;\x03⁢",
			// iukcy;[і] iuml;[ï] iuml[ï].
			"\x04kcy;\x02і\x03ml;\x02ï\x02ml\x02ï",
			// jcirc;[ĵ] jcy;[й].
			"\x04irc;\x02ĵ\x02y;\x02й",
			// jfr;[𝔧].
			"\x02r;\x04𝔧",
			// jmath;[ȷ].
			"\x04ath;\x02ȷ",
			// jopf;[𝕛].
			"\x03pf;\x04𝕛",
			// jsercy;[ј] jscr;[𝒿].
			"\x05ercy;\x02ј\x03cr;\x04𝒿",
			// jukcy;[є].
			"\x04kcy;\x02є",
			// kappav;[ϰ] kappa;[κ].
			"\x05ppav;\x02ϰ\x04ppa;\x02κ",
			// kcedil;[ķ] kcy;[к].
			"\x05edil;\x02ķ\x02y;\x02к",
			// kfr;[𝔨].
			"\x02r;\x04𝔨",
			// kgreen;[ĸ].
			"\x05reen;\x02ĸ",
			// khcy;[х].
			"\x03cy;\x02х",
			// kjcy;[ќ].
			"\x03cy;\x02ќ",
			// kopf;[𝕜].
			"\x03pf;\x04𝕜",
			// kscr;[𝓀].
			"\x03cr;\x04𝓀",
			// lAtail;[⤛] lAarr;[⇚] lArr;[⇐].
			"\x05tail;\x03⤛\x04arr;\x03⇚\x03rr;\x03⇐",
			// lBarr;[⤎].
			"\x04arr;\x03⤎",
			// lEg;[⪋] lE;[≦].
			"\x02g;\x03⪋\x01;\x03≦",
			// lHar;[⥢].
			"\x03ar;\x03⥢",
			// laemptyv;[⦴] larrbfs;[⤟] larrsim;[⥳] lacute;[ĺ] lagran;[ℒ] lambda;[λ] langle;[⟨] larrfs;[⤝] larrhk;[↩] larrlp;[↫] larrpl;[⤹] larrtl;[↢] latail;[⤙] langd;[⦑] laquo;[«] larrb;[⇤] lates;[⪭︀] lang;[⟨] laquo[«] larr;[←] late;[⪭] lap;[⪅] lat;[⪫].
			"\x07emptyv;\x03⦴\x06rrbfs;\x03⤟\x06rrsim;\x03⥳\x05cute;\x02ĺ\x05gran;\x03ℒ\x05mbda;\x02λ\x05ngle;\x03⟨\x05rrfs;\x03⤝\x05rrhk;\x03↩\x05rrlp;\x03↫\x05rrpl;\x03⤹\x05rrtl;\x03↢\x05tail;\x03⤙\x04ngd;\x03⦑\x04quo;\x02«\x04rrb;\x03⇤\x04tes;\x06⪭︀\x03ng;\x03⟨\x03quo\x02«\x03rr;\x03←\x03te;\x03⪭\x02p;\x03⪅\x02t;\x03⪫",
			// lbrksld;[⦏] lbrkslu;[⦍] lbrace;[{] lbrack;[[] lbarr;[⤌] lbbrk;[❲] lbrke;[⦋].
			"\x06rksld;\x03⦏\x06rkslu;\x03⦍\x05race;\x01{\x05rack;\x01[\x04arr;\x03⤌\x04brk;\x03❲\x04rke;\x03⦋",
			// lcaron;[ľ] lcedil;[ļ] lceil;[⌈] lcub;[{] lcy;[л].
			"\x05aron;\x02ľ\x05edil;\x02ļ\x04eil;\x03⌈\x03ub;\x01{\x02y;\x02л",
			// ldrushar;[⥋] ldrdhar;[⥧] ldquor;[„] ldquo;[“] ldca;[⤶] ldsh;[↲].
			"\x07rushar;\x03⥋\x06rdhar;\x03⥧\x05quor;\x03„\x04quo;\x03“\x03ca;\x03⤶\x03sh;\x03↲",
			// leftrightsquigarrow;[↭] leftrightharpoons;[⇋] leftharpoondown;[↽] leftrightarrows;[⇆] leftleftarrows;[⇇] leftrightarrow;[↔] leftthreetimes;[⋋] leftarrowtail;[↢] leftharpoonup;[↼] lessapprox;[⪅] lesseqqgtr;[⪋] leftarrow;[←] lesseqgtr;[⋚] leqslant;[⩽] lesdotor;[⪃] lesdoto;[⪁] lessdot;[⋖] lessgtr;[≶] lesssim;[≲] lesdot;[⩿] lesges;[⪓] lescc;[⪨] leqq;[≦] lesg;[⋚︀] leg;[⋚] leq;[≤] les;[⩽] le;[≤].
			"\x12ftrightsquigarrow;\x03↭\x10ftrightharpoons;\x03⇋\x0eftharpoondown;\x03↽\x0eftrightarrows;\x03⇆\x0dftleftarrows;\x03⇇\x0dftrightarrow;\x03↔\x0dftthreetimes;\x03⋋\x0cftarrowtail;\x03↢\x0cftharpoonup;\x03↼\x09ssapprox;\x03⪅\x09sseqqgtr;\x03⪋\x08ftarrow;\x03←\x08sseqgtr;\x03⋚\x07qslant;\x03⩽\x07sdotor;\x03⪃\x06sdoto;\x03⪁\x06ssdot;\x03⋖\x06ssgtr;\x03≶\x06sssim;\x03≲\x05sdot;\x03⩿\x05sges;\x03⪓\x04scc;\x03⪨\x03qq;\x03≦\x03sg;\x06⋚︀\x02g;\x03⋚\x02q;\x03≤\x02s;\x03⩽\x01;\x03≤",
			// lfisht;[⥼] lfloor;[⌊] lfr;[𝔩].
			"\x05isht;\x03⥼\x05loor;\x03⌊\x02r;\x04𝔩",
			// lgE;[⪑] lg;[≶].
			"\x02E;\x03⪑\x01;\x03≶",
			// lharul;[⥪] lhard;[↽] lharu;[↼] lhblk;[▄].
			"\x05arul;\x03⥪\x04ard;\x03↽\x04aru;\x03↼\x04blk;\x03▄",
			// ljcy;[љ].
			"\x03cy;\x02љ",
			// llcorner;[⌞] llhard;[⥫] llarr;[⇇] lltri;[◺] ll;[≪].
			"\x07corner;\x03⌞\x05hard;\x03⥫\x04arr;\x03⇇\x04tri;\x03◺\x01;\x03≪",
			// lmoustache;[⎰] lmidot;[ŀ] lmoust;[⎰].
			"\x09oustache;\x03⎰\x05idot;\x02ŀ\x05oust;\x03⎰",
			// lnapprox;[⪉] lneqq;[≨] lnsim;[⋦] lnap;[⪉] lneq;[⪇] lnE;[≨] lne;[⪇].
			"\x07approx;\x03⪉\x04eqq;\x03≨\x04sim;\x03⋦\x03ap;\x03⪉\x03eq;\x03⪇\x02E;\x03≨\x02e;\x03⪇",
			// longleftrightarrow;[⟷] longrightarrow;[⟶] looparrowright;[↬] longleftarrow;[⟵] looparrowleft;[↫] longmapsto;[⟼] lotimes;[⨴] lozenge;[◊] loplus;[⨭] lowast;[∗] lowbar;[_] loang;[⟬] loarr;[⇽] lobrk;[⟦] lopar;[⦅] lopf;[𝕝] lozf;[⧫] loz;[◊].
			"\x11ngleftrightarrow;\x03⟷\x0dngrightarrow;\x03⟶\x0doparrowright;\x03↬\x0cngleftarrow;\x03⟵\x0coparrowleft;\x03↫\x09ngmapsto;\x03⟼\x06times;\x03⨴\x06zenge;\x03◊\x05plus;\x03⨭\x05wast;\x03∗\x05wbar;\x01_\x04ang;\x03⟬\x04arr;\x03⇽\x04brk;\x03⟦\x04par;\x03⦅\x03pf;\x04𝕝\x03zf;\x03⧫\x02z;\x03◊",
			// lparlt;[⦓] lpar;[(].
			"\x05arlt;\x03⦓\x03ar;\x01(",
			// lrcorner;[⌟] lrhard;[⥭] lrarr;[⇆] lrhar;[⇋] lrtri;[⊿] lrm;[‎].
			"\x07corner;\x03⌟\x05hard;\x03⥭\x04arr;\x03⇆\x04har;\x03⇋\x04tri;\x03⊿\x02m;\x03‎",
			// lsaquo;[‹] lsquor;[‚] lstrok;[ł] lsime;[⪍] lsimg;[⪏] lsquo;[‘] lscr;[𝓁] lsim;[≲] lsqb;[[] lsh;[↰].
			"\x05aquo;\x03‹\x05quor;\x03‚\x05trok;\x02ł\x04ime;\x03⪍\x04img;\x03⪏\x04quo;\x03‘\x03cr;\x04𝓁\x03im;\x03≲\x03qb;\x01[\x02h;\x03↰",
			// ltquest;[⩻] lthree;[⋋] ltimes;[⋉] ltlarr;[⥶] ltrPar;[⦖] ltcir;[⩹] ltdot;[⋖] ltrie;[⊴] ltrif;[◂] ltcc;[⪦] ltri;[◃] lt;[<].
			"\x06quest;\x03⩻\x05hree;\x03⋋\x05imes;\x03⋉\x05larr;\x03⥶\x05rPar;\x03⦖\x04cir;\x03⩹\x04dot;\x03⋖\x04rie;\x03⊴\x04rif;\x03◂\x03cc;\x03⪦\x03ri;\x03◃\x01;\x01<",
			// lurdshar;[⥊] luruhar;[⥦].
			"\x07rdshar;\x03⥊\x06ruhar;\x03⥦",
			// lvertneqq;[≨︀] lvnE;[≨︀].
			"\x08ertneqq;\x06≨︀\x03nE;\x06≨︀",
			// mDDot;[∺].
			"\x04Dot;\x03∺",
			// mapstodown;[↧] mapstoleft;[↤] mapstoup;[↥] maltese;[✠] mapsto;[↦] marker;[▮] macr;[¯] male;[♂] malt;[✠] macr[¯] map;[↦].
			"\x09pstodown;\x03↧\x09pstoleft;\x03↤\x07pstoup;\x03↥\x06ltese;\x03✠\x05psto;\x03↦\x05rker;\x03▮\x03cr;\x02¯\x03le;\x03♂\x03lt;\x03✠\x02cr\x02¯\x02p;\x03↦",
			// mcomma;[⨩] mcy;[м].
			"\x05omma;\x03⨩\x02y;\x02м",
			// mdash;[—].
			"\x04ash;\x03—",
			// measuredangle;[∡].
			"\x0casuredangle;\x03∡",
			// mfr;[𝔪].
			"\x02r;\x04𝔪",
			// mho;[℧].
			"\x02o;\x03℧",
			// minusdu;[⨪] midast;[*] midcir;[⫰] middot;[·] minusb;[⊟] minusd;[∸] micro;[µ] middot[·] minus;[−] micro[µ] mid;[∣].
			"\x06nusdu;\x03⨪\x05dast;\x01*\x05dcir;\x03⫰\x05ddot;\x02·\x05nusb;\x03⊟\x05nusd;\x03∸\x04cro;\x02µ\x04ddot\x02·\x04nus;\x03−\x03cro\x02µ\x02d;\x03∣",
			// mlcp;[⫛] mldr;[…].
			"\x03cp;\x03⫛\x03dr;\x03…",
			// mnplus;[∓].
			"\x05plus;\x03∓",
			// models;[⊧] mopf;[𝕞].
			"\x05dels;\x03⊧\x03pf;\x04𝕞",
			// mp;[∓].
			"\x01;\x03∓",
			// mstpos;[∾] mscr;[𝓂].
			"\x05tpos;\x03∾\x03cr;\x04𝓂",
			// multimap;[⊸] mumap;[⊸] mu;[μ].
			"\x07ltimap;\x03⊸\x04map;\x03⊸\x01;\x02μ",
			// nGtv;[≫̸] nGg;[⋙̸] nGt;[≫⃒].
			"\x03tv;\x05≫̸\x02g;\x05⋙̸\x02t;\x06≫⃒",
			// nLeftrightarrow;[⇎] nLeftarrow;[⇍] nLtv;[≪̸] nLl;[⋘̸] nLt;[≪⃒].
			"\x0eeftrightarrow;\x03⇎\x09eftarrow;\x03⇍\x03tv;\x05≪̸\x02l;\x05⋘̸\x02t;\x06≪⃒",
			// nRightarrow;[⇏].
			"\x0aightarrow;\x03⇏",
			// nVDash;[⊯] nVdash;[⊮].
			"\x05Dash;\x03⊯\x05dash;\x03⊮",
			// naturals;[ℕ] napprox;[≉] natural;[♮] nacute;[ń] nabla;[∇] napid;[≋̸] napos;[ŉ] natur;[♮] nang;[∠⃒] napE;[⩰̸] nap;[≉].
			"\x07turals;\x03ℕ\x06pprox;\x03≉\x06tural;\x03♮\x05cute;\x02ń\x04bla;\x03∇\x04pid;\x05≋̸\x04pos;\x02ŉ\x04tur;\x03♮\x03ng;\x06∠⃒\x03pE;\x05⩰̸\x02p;\x03≉",
			// nbumpe;[≏̸] nbump;[≎̸] nbsp;[ ] nbsp[ ].
			"\x05umpe;\x05≏̸\x04ump;\x05≎̸\x03sp;\x02 \x02sp\x02 ",
			// ncongdot;[⩭̸] ncaron;[ň] ncedil;[ņ] ncong;[≇] ncap;[⩃] ncup;[⩂] ncy;[н].
			"\x07ongdot;\x05⩭̸\x05aron;\x02ň\x05edil;\x02ņ\x04ong;\x03≇\x03ap;\x03⩃\x03up;\x03⩂\x02y;\x02н",
			// ndash;[–].
			"\x04ash;\x03–",
			// nearrow;[↗] nexists;[∄] nearhk;[⤤] nequiv;[≢] nesear;[⤨] nexist;[∄] neArr;[⇗] nearr;[↗] nedot;[≐̸] nesim;[≂̸] ne;[≠].
			"\x06arrow;\x03↗\x06xists;\x03∄\x05arhk;\x03⤤\x05quiv;\x03≢\x05sear;\x03⤨\x05xist;\x03∄\x04Arr;\x03⇗\x04arr;\x03↗\x04dot;\x05≐̸\x04sim;\x05≂̸\x01;\x03≠",
			// nfr;[𝔫].
			"\x02r;\x04𝔫",
			// ngeqslant;[⩾̸] ngeqq;[≧̸] ngsim;[≵] ngeq;[≱] nges;[⩾̸] ngtr;[≯] ngE;[≧̸] nge;[≱] ngt;[≯].
			"\x08eqslant;\x05⩾̸\x04eqq;\x05≧̸\x04sim;\x03≵\x03eq;\x03≱\x03es;\x05⩾̸\x03tr;\x03≯\x02E;\x05≧̸\x02e;\x03≱\x02t;\x03≯",
			// nhArr;[⇎] nharr;[↮] nhpar;[⫲].
			"\x04Arr;\x03⇎\x04arr;\x03↮\x04par;\x03⫲",
			// nisd;[⋺] nis;[⋼] niv;[∋] ni;[∋].
			"\x03sd;\x03⋺\x02s;\x03⋼\x02v;\x03∋\x01;\x03∋",
			// njcy;[њ].
			"\x03cy;\x02њ",
			// nleftrightarrow;[↮] nleftarrow;[↚] nleqslant;[⩽̸] nltrie;[⋬] nlArr;[⇍] nlarr;[↚] nleqq;[≦̸] nless;[≮] nlsim;[≴] nltri;[⋪] nldr;[‥] nleq;[≰] nles;[⩽̸] nlE;[≦̸] nle;[≰] nlt;[≮].
			"\x0eeftrightarrow;\x03↮\x09eftarrow;\x03↚\x08eqslant;\x05⩽̸\x05trie;\x03⋬\x04Arr;\x03⇍\x04arr;\x03↚\x04eqq;\x05≦̸\x04ess;\x03≮\x04sim;\x03≴\x04tri;\x03⋪\x03dr;\x03‥\x03eq;\x03≰\x03es;\x05⩽̸\x02E;\x05≦̸\x02e;\x03≰\x02t;\x03≮",
			// nmid;[∤].
			"\x03id;\x03∤",
			// notindot;[⋵̸] notinva;[∉] notinvb;[⋷] notinvc;[⋶] notniva;[∌] notnivb;[⋾] notnivc;[⋽] notinE;[⋹̸] notin;[∉] notni;[∌] nopf;[𝕟] not;[¬] not[¬].
			"\x07tindot;\x05⋵̸\x06tinva;\x03∉\x06tinvb;\x03⋷\x06tinvc;\x03⋶\x06tniva;\x03∌\x06tnivb;\x03⋾\x06tnivc;\x03⋽\x05tinE;\x05⋹̸\x04tin;\x03∉\x04tni;\x03∌\x03pf;\x04𝕟\x02t;\x02¬\x01t\x02¬",
			// nparallel;[∦] npolint;[⨔] npreceq;[⪯̸] nparsl;[⫽⃥] nprcue;[⋠] npart;[∂̸] nprec;[⊀] npar;[∦] npre;[⪯̸] npr;[⊀].
			"\x08arallel;\x03∦\x06olint;\x03⨔\x06receq;\x05⪯̸\x05arsl;\x06⫽⃥\x05rcue;\x03⋠\x04art;\x05∂̸\x04rec;\x03⊀\x03ar;\x03∦\x03re;\x05⪯̸\x02r;\x03⊀",
			// nrightarrow;[↛] nrarrc;[⤳̸] nrarrw;[↝̸] nrtrie;[⋭] nrArr;[⇏] nrarr;[↛] nrtri;[⋫].
			"\x0aightarrow;\x03↛\x05arrc;\x05⤳̸\x05arrw;\x05↝̸\x05trie;\x03⋭\x04Arr;\x03⇏\x04arr;\x03↛\x04tri;\x03⋫",
			// nshortparallel;[∦] nsubseteqq;[⫅̸] nsupseteqq;[⫆̸] nshortmid;[∤] nsubseteq;[⊈] nsupseteq;[⊉] nsqsube;[⋢] nsqsupe;[⋣] nsubset;[⊂⃒] nsucceq;[⪰̸] nsupset;[⊃⃒] nsccue;[⋡] nsimeq;[≄] nsime;[≄] nsmid;[∤] nspar;[∦] nsubE;[⫅̸] nsube;[⊈] nsucc;[⊁] nsupE;[⫆̸] nsupe;[⊉] nsce;[⪰̸] nscr;[𝓃] nsim;[≁] nsub;[⊄] nsup;[⊅] nsc;[⊁].
			"\x0dhortparallel;\x03∦\x09ubseteqq;\x05⫅̸\x09upseteqq;\x05⫆̸\x08hortmid;\x03∤\x08ubseteq;\x03⊈\x08upseteq;\x03⊉\x06qsube;\x03⋢\x06qsupe;\x03⋣\x06ubset;\x06⊂⃒\x06ucceq;\x05⪰̸\x06upset;\x06⊃⃒\x05ccue;\x03⋡\x05imeq;\x03≄\x04ime;\x03≄\x04mid;\x03∤\x04par;\x03∦\x04ubE;\x05⫅̸\x04ube;\x03⊈\x04ucc;\x03⊁\x04upE;\x05⫆̸\x04upe;\x03⊉\x03ce;\x05⪰̸\x03cr;\x04𝓃\x03im;\x03≁\x03ub;\x03⊄\x03up;\x03⊅\x02c;\x03⊁",
			// ntrianglerighteq;[⋭] ntrianglelefteq;[⋬] ntriangleright;[⋫] ntriangleleft;[⋪] ntilde;[ñ] ntilde[ñ] ntgl;[≹] ntlg;[≸].
			"\x0frianglerighteq;\x03⋭\x0erianglelefteq;\x03⋬\x0driangleright;\x03⋫\x0criangleleft;\x03⋪\x05ilde;\x02ñ\x04ilde\x02ñ\x03gl;\x03≹\x03lg;\x03≸",
			// numero;[№] numsp;[ ] num;[#] nu;[ν].
			"\x05mero;\x03№\x04msp;\x03 \x02m;\x01#\x01;\x02ν",
			// nvinfin;[⧞] nvltrie;[⊴⃒] nvrtrie;[⊵⃒] nvDash;[⊭] nvHarr;[⤄] nvdash;[⊬] nvlArr;[⤂] nvrArr;[⤃] nvsim;[∼⃒] nvap;[≍⃒] nvge;[≥⃒] nvgt;[>⃒] nvle;[≤⃒] nvlt;[<⃒].
			"\x06infin;\x03⧞\x06ltrie;\x06⊴⃒\x06rtrie;\x06⊵⃒\x05Dash;\x03⊭\x05Harr;\x03⤄\x05dash;\x03⊬\x05lArr;\x03⤂\x05rArr;\x03⤃\x04sim;\x06∼⃒\x03ap;\x06≍⃒\x03ge;\x06≥⃒\x03gt;\x04>⃒\x03le;\x06≤⃒\x03lt;\x04<⃒",
			// nwarrow;[↖] nwarhk;[⤣] nwnear;[⤧] nwArr;[⇖] nwarr;[↖].
			"\x06arrow;\x03↖\x05arhk;\x03⤣\x05near;\x03⤧\x04Arr;\x03⇖\x04arr;\x03↖",
			// oS;[Ⓢ].
			"\x01;\x03Ⓢ",
			// oacute;[ó] oacute[ó] oast;[⊛].
			"\x05cute;\x02ó\x04cute\x02ó\x03st;\x03⊛",
			// ocirc;[ô] ocir;[⊚] ocirc[ô] ocy;[о].
			"\x04irc;\x02ô\x03ir;\x03⊚\x03irc\x02ô\x02y;\x02о",
			// odblac;[ő] odsold;[⦼] odash;[⊝] odiv;[⨸] odot;[⊙].
			"\x05blac;\x02ő\x05sold;\x03⦼\x04ash;\x03⊝\x03iv;\x03⨸\x03ot;\x03⊙",
			// oelig;[œ].
			"\x04lig;\x02œ",
			// ofcir;[⦿] ofr;[𝔬].
			"\x04cir;\x03⦿\x02r;\x04𝔬",
			// ograve;[ò] ograve[ò] ogon;[˛] ogt;[⧁].
			"\x05rave;\x02ò\x04rave\x02ò\x03on;\x02˛\x02t;\x03⧁",
			// ohbar;[⦵] ohm;[Ω].
			"\x04bar;\x03⦵\x02m;\x02Ω",
			// oint;[∮].
			"\x03nt;\x03∮",
			// olcross;[⦻] olarr;[↺] olcir;[⦾] oline;[‾] olt;[⧀].
			"\x06cross;\x03⦻\x04arr;\x03↺\x04cir;\x03⦾\x04ine;\x03‾\x02t;\x03⧀",
			// omicron;[ο] ominus;[⊖] omacr;[ō] omega;[ω] omid;[⦶].
			"\x06icron;\x02ο\x05inus;\x03⊖\x04acr;\x02ō\x04ega;\x02ω\x03id;\x03⦶",
			// oopf;[𝕠].
			"\x03pf;\x04𝕠",
			// operp;[⦹] oplus;[⊕] opar;[⦷].
			"\x04erp;\x03⦹\x04lus;\x03⊕\x03ar;\x03⦷",
			// orderof;[ℴ] orslope;[⩗] origof;[⊶] orarr;[↻] order;[ℴ] ordf;[ª] ordm;[º] oror;[⩖] ord;[⩝] ordf[ª] ordm[º] orv;[⩛] or;[∨].
			"\x06derof;\x03ℴ\x06slope;\x03⩗\x05igof;\x03⊶\x04arr;\x03↻\x04der;\x03ℴ\x03df;\x02ª\x03dm;\x02º\x03or;\x03⩖\x02d;\x03⩝\x02df\x02ª\x02dm\x02º\x02v;\x03⩛\x01;\x03∨",
			// oslash;[ø] oslash[ø] oscr;[ℴ] osol;[⊘].
			"\x05lash;\x02ø\x04lash\x02ø\x03cr;\x03ℴ\x03ol;\x03⊘",
			// otimesas;[⨶] otilde;[õ] otimes;[⊗] otilde[õ].
			"\x07imesas;\x03⨶\x05ilde;\x02õ\x05imes;\x03⊗\x04ilde\x02õ",
			// ouml;[ö] ouml[ö].
			"\x03ml;\x02ö\x02ml\x02ö",
			// ovbar;[⌽].
			"\x04bar;\x03⌽",
			// parallel;[∥] parsim;[⫳] parsl;[⫽] para;[¶] part;[∂] par;[∥] para[¶].
			"\x07rallel;\x03∥\x05rsim;\x03⫳\x04rsl;\x03⫽\x03ra;\x02¶\x03rt;\x03∂\x02r;\x03∥\x02ra\x02¶",
			// pcy;[п].
			"\x02y;\x02п",
			// pertenk;[‱] percnt;[%] period;[.] permil;[‰] perp;[⊥].
			"\x06rtenk;\x03‱\x05rcnt;\x01%\x05riod;\x01.\x05rmil;\x03‰\x03rp;\x03⊥",
			// pfr;[𝔭].
			"\x02r;\x04𝔭",
			// phmmat;[ℳ] phone;[☎] phiv;[ϕ] phi;[φ].
			"\x05mmat;\x03ℳ\x04one;\x03☎\x03iv;\x02ϕ\x02i;\x02φ",
			// pitchfork;[⋔] piv;[ϖ] pi;[π].
			"\x08tchfork;\x03⋔\x02v;\x02ϖ\x01;\x02π",
			// plusacir;[⨣] planckh;[ℎ] pluscir;[⨢] plussim;[⨦] plustwo;[⨧] planck;[ℏ] plankv;[ℏ] plusdo;[∔] plusdu;[⨥] plusmn;[±] plusb;[⊞] pluse;[⩲] plusmn[±] plus;[+].
			"\x07usacir;\x03⨣\x06anckh;\x03ℎ\x06uscir;\x03⨢\x06ussim;\x03⨦\x06ustwo;\x03⨧\x05anck;\x03ℏ\x05ankv;\x03ℏ\x05usdo;\x03∔\x05usdu;\x03⨥\x05usmn;\x02±\x04usb;\x03⊞\x04use;\x03⩲\x04usmn\x02±\x03us;\x01+",
			// pm;[±].
			"\x01;\x02±",
			// pointint;[⨕] pound;[£] popf;[𝕡] pound[£].
			"\x07intint;\x03⨕\x04und;\x02£\x03pf;\x04𝕡\x03und\x02£",
			// preccurlyeq;[≼] precnapprox;[⪹] precapprox;[⪷] precneqq;[⪵] precnsim;[⋨] profalar;[⌮] profline;[⌒] profsurf;[⌓] precsim;[≾] preceq;[⪯] primes;[ℙ] prnsim;[⋨] propto;[∝] prurel;[⊰] prcue;[≼] prime;[′] prnap;[⪹] prsim;[≾] prap;[⪷] prec;[≺] prnE;[⪵] prod;[∏] prop;[∝] prE;[⪳] pre;[⪯] pr;[≺].
			"\x0aeccurlyeq;\x03≼\x0aecnapprox;\x03⪹\x09ecapprox;\x03⪷\x07ecneqq;\x03⪵\x07ecnsim;\x03⋨\x07ofalar;\x03⌮\x07ofline;\x03⌒\x07ofsurf;\x03⌓\x06ecsim;\x03≾\x05eceq;\x03⪯\x05imes;\x03ℙ\x05nsim;\x03⋨\x05opto;\x03∝\x05urel;\x03⊰\x04cue;\x03≼\x04ime;\x03′\x04nap;\x03⪹\x04sim;\x03≾\x03ap;\x03⪷\x03ec;\x03≺\x03nE;\x03⪵\x03od;\x03∏\x03op;\x03∝\x02E;\x03⪳\x02e;\x03⪯\x01;\x03≺",
			// pscr;[𝓅] psi;[ψ].
			"\x03cr;\x04𝓅\x02i;\x02ψ",
			// puncsp;[ ].
			"\x05ncsp;\x03 ",
			// qfr;[𝔮].
			"\x02r;\x04𝔮",
			// qint;[⨌].
			"\x03nt;\x03⨌",
			// qopf;[𝕢].
			"\x03pf;\x04𝕢",
			// qprime;[⁗].
			"\x05rime;\x03⁗",
			// qscr;[𝓆].
			"\x03cr;\x04𝓆",
			// quaternions;[ℍ] quatint;[⨖] questeq;[≟] quest;[?] quot;[\"] quot[\"].
			"\x0aaternions;\x03ℍ\x06atint;\x03⨖\x06esteq;\x03≟\x04est;\x01?\x03ot;\x01\"\x02ot\x01\"",
			// rAtail;[⤜] rAarr;[⇛] rArr;[⇒].
			"\x05tail;\x03⤜\x04arr;\x03⇛\x03rr;\x03⇒",
			// rBarr;[⤏].
			"\x04arr;\x03⤏",
			// rHar;[⥤].
			"\x03ar;\x03⥤",
			// rationals;[ℚ] raemptyv;[⦳] rarrbfs;[⤠] rarrsim;[⥴] racute;[ŕ] rangle;[⟩] rarrap;[⥵] rarrfs;[⤞] rarrhk;[↪] rarrlp;[↬] rarrpl;[⥅] rarrtl;[↣] ratail;[⤚] radic;[√] rangd;[⦒] range;[⦥] raquo;[»] rarrb;[⇥] rarrc;[⤳] rarrw;[↝] ratio;[∶] race;[∽̱] rang;[⟩] raquo[»] rarr;[→].
			"\x08tionals;\x03ℚ\x07emptyv;\x03⦳\x06rrbfs;\x03⤠\x06rrsim;\x03⥴\x05cute;\x02ŕ\x05ngle;\x03⟩\x05rrap;\x03⥵\x05rrfs;\x03⤞\x05rrhk;\x03↪\x05rrlp;\x03↬\x05rrpl;\x03⥅\x05rrtl;\x03↣\x05tail;\x03⤚\x04dic;\x03√\x04ngd;\x03⦒\x04nge;\x03⦥\x04quo;\x02»\x04rrb;\x03⇥\x04rrc;\x03⤳\x04rrw;\x03↝\x04tio;\x03∶\x03ce;\x05∽̱\x03ng;\x03⟩\x03quo\x02»\x03rr;\x03→",
			// rbrksld;[⦎] rbrkslu;[⦐] rbrace;[}] rbrack;[]] rbarr;[⤍] rbbrk;[❳] rbrke;[⦌].
			"\x06rksld;\x03⦎\x06rkslu;\x03⦐\x05race;\x01}\x05rack;\x01]\x04arr;\x03⤍\x04brk;\x03❳\x04rke;\x03⦌",
			// rcaron;[ř] rcedil;[ŗ] rceil;[⌉] rcub;[}] rcy;[р].
			"\x05aron;\x02ř\x05edil;\x02ŗ\x04eil;\x03⌉\x03ub;\x01}\x02y;\x02р",
			// rdldhar;[⥩] rdquor;[”] rdquo;[”] rdca;[⤷] rdsh;[↳].
			"\x06ldhar;\x03⥩\x05quor;\x03”\x04quo;\x03”\x03ca;\x03⤷\x03sh;\x03↳",
			// realpart;[ℜ] realine;[ℛ] reals;[ℝ] real;[ℜ] rect;[▭] reg;[®] reg[®].
			"\x07alpart;\x03ℜ\x06aline;\x03ℛ\x04als;\x03ℝ\x03al;\x03ℜ\x03ct;\x03▭\x02g;\x02®\x01g\x02®",
			// rfisht;[⥽] rfloor;[⌋] rfr;[𝔯].
			"\x05isht;\x03⥽\x05loor;\x03⌋\x02r;\x04𝔯",
			// rharul;[⥬] rhard;[⇁] rharu;[⇀] rhov;[ϱ] rho;[ρ].
			"\x05arul;\x03⥬\x04ard;\x03⇁\x04aru;\x03⇀\x03ov;\x02ϱ\x02o;\x02ρ",
			// rightleftharpoons;[⇌] rightharpoondown;[⇁] rightrightarrows;[⇉] rightleftarrows;[⇄] rightsquigarrow;[↝] rightthreetimes;[⋌] rightarrowtail;[↣] rightharpoonup;[⇀] risingdotseq;[≓] rightarrow;[→] ring;[˚].
			"\x10ghtleftharpoons;\x03⇌\x0fghtharpoondown;\x03⇁\x0fghtrightarrows;\x03⇉\x0eghtleftarrows;\x03⇄\x0eghtsquigarrow;\x03↝\x0eghtthreetimes;\x03⋌\x0dghtarrowtail;\x03↣\x0dghtharpoonup;\x03⇀\x0bsingdotseq;\x03≓\x09ghtarrow;\x03→\x03ng;\x02˚",
			// rlarr;[⇄] rlhar;[⇌] rlm;[‏].
			"\x04arr;\x03⇄\x04har;\x03⇌\x02m;\x03‏",
			// rmoustache;[⎱] rmoust;[⎱].
			"\x09oustache;\x03⎱\x05oust;\x03⎱",
			// rnmid;[⫮].
			"\x04mid;\x03⫮",
			// rotimes;[⨵] roplus;[⨮] roang;[⟭] roarr;[⇾] robrk;[⟧] ropar;[⦆] ropf;[𝕣].
			"\x06times;\x03⨵\x05plus;\x03⨮\x04ang;\x03⟭\x04arr;\x03⇾\x04brk;\x03⟧\x04par;\x03⦆\x03pf;\x04𝕣",
			// rppolint;[⨒] rpargt;[⦔] rpar;[)].
			"\x07polint;\x03⨒\x05argt;\x03⦔\x03ar;\x01)",
			// rrarr;[⇉].
			"\x04arr;\x03⇉",
			// rsaquo;[›] rsquor;[’] rsquo;[’] rscr;[𝓇] rsqb;[]] rsh;[↱].
			"\x05aquo;\x03›\x05quor;\x03’\x04quo;\x03’\x03cr;\x04𝓇\x03qb;\x01]\x02h;\x03↱",
			// rtriltri;[⧎] rthree;[⋌] rtimes;[⋊] rtrie;[⊵] rtrif;[▸] rtri;[▹].
			"\x07riltri;\x03⧎\x05hree;\x03⋌\x05imes;\x03⋊\x04rie;\x03⊵\x04rif;\x03▸\x03ri;\x03▹",
			// ruluhar;[⥨].
			"\x06luhar;\x03⥨",
			// rx;[℞].
			"\x01;\x03℞",
			// sacute;[ś].
			"\x05cute;\x02ś",
			// sbquo;[‚].
			"\x04quo;\x03‚",
			// scpolint;[⨓] scaron;[š] scedil;[ş] scnsim;[⋩] sccue;[≽] scirc;[ŝ] scnap;[⪺] scsim;[≿] scap;[⪸] scnE;[⪶] scE;[⪴] sce;[⪰] scy;[с] sc;[≻].
			"\x07polint;\x03⨓\x05aron;\x02š\x05edil;\x02ş\x05nsim;\x03⋩\x04cue;\x03≽\x04irc;\x02ŝ\x04nap;\x03⪺\x04sim;\x03≿\x03ap;\x03⪸\x03nE;\x03⪶\x02E;\x03⪴\x02e;\x03⪰\x02y;\x02с\x01;\x03≻",
			// sdotb;[⊡] sdote;[⩦] sdot;[⋅].
			"\x04otb;\x03⊡\x04ote;\x03⩦\x03ot;\x03⋅",
			// setminus;[∖] searrow;[↘] searhk;[⤥] seswar;[⤩] seArr;[⇘] searr;[↘] setmn;[∖] sect;[§] semi;[;] sext;[✶] sect[§].
			"\x07tminus;\x03∖\x06arrow;\x03↘\x05arhk;\x03⤥\x05swar;\x03⤩\x04Arr;\x03⇘\x04arr;\x03↘\x04tmn;\x03∖\x03ct;\x02§\x03mi;\x01;\x03xt;\x03✶\x02ct\x02§",
			// sfrown;[⌢] sfr;[𝔰].
			"\x05rown;\x03⌢\x02r;\x04𝔰",
			// shortparallel;[∥] shortmid;[∣] shchcy;[щ] sharp;[♯] shcy;[ш] shy;[­] shy[­].
			"\x0cortparallel;\x03∥\x07ortmid;\x03∣\x05chcy;\x02щ\x04arp;\x03♯\x03cy;\x02ш\x02y;\x02­\x01y\x02­",
			// simplus;[⨤] simrarr;[⥲] sigmaf;[ς] sigmav;[ς] simdot;[⩪] sigma;[σ] simeq;[≃] simgE;[⪠] simlE;[⪟] simne;[≆] sime;[≃] simg;[⪞] siml;[⪝] sim;[∼].
			"\x06mplus;\x03⨤\x06mrarr;\x03⥲\x05gmaf;\x02ς\x05gmav;\x02ς\x05mdot;\x03⩪\x04gma;\x02σ\x04meq;\x03≃\x04mgE;\x03⪠\x04mlE;\x03⪟\x04mne;\x03≆\x03me;\x03≃\x03mg;\x03⪞\x03ml;\x03⪝\x02m;\x03∼",
			// slarr;[←].
			"\x04arr;\x03←",
			// smallsetminus;[∖] smeparsl;[⧤] smashp;[⨳] smile;[⌣] smtes;[⪬︀] smid;[∣] smte;[⪬] smt;[⪪].
			"\x0callsetminus;\x03∖\x07eparsl;\x03⧤\x05ashp;\x03⨳\x04ile;\x03⌣\x04tes;\x06⪬︀\x03id;\x03∣\x03te;\x03⪬\x02t;\x03⪪",
			// softcy;[ь] solbar;[⌿] solb;[⧄] sopf;[𝕤] sol;[/].
			"\x05ftcy;\x02ь\x05lbar;\x03⌿\x03lb;\x03⧄\x03pf;\x04𝕤\x02l;\x01/",
			// spadesuit;[♠] spades;[♠] spar;[∥].
			"\x08adesuit;\x03♠\x05ades;\x03♠\x03ar;\x03∥",
			// sqsubseteq;[⊑] sqsupseteq;[⊒] sqsubset;[⊏] sqsupset;[⊐] sqcaps;[⊓︀] sqcups;[⊔︀] sqsube;[⊑] sqsupe;[⊒] square;[□] squarf;[▪] sqcap;[⊓] sqcup;[⊔] sqsub;[⊏] sqsup;[⊐] squf;[▪] squ;[□].
			"\x09subseteq;\x03⊑\x09supseteq;\x03⊒\x07subset;\x03⊏\x07supset;\x03⊐\x05caps;\x06⊓︀\x05cups;\x06⊔︀\x05sube;\x03⊑\x05supe;\x03⊒\x05uare;\x03□\x05uarf;\x03▪\x04cap;\x03⊓\x04cup;\x03⊔\x04sub;\x03⊏\x04sup;\x03⊐\x03uf;\x03▪\x02u;\x03□",
			// srarr;[→].
			"\x04arr;\x03→",
			// ssetmn;[∖] ssmile;[⌣] sstarf;[⋆] sscr;[𝓈].
			"\x05etmn;\x03∖\x05mile;\x03⌣\x05tarf;\x03⋆\x03cr;\x04𝓈",
			// straightepsilon;[ϵ] straightphi;[ϕ] starf;[★] strns;[¯] star;[☆].
			"\x0eraightepsilon;\x02ϵ\x0araightphi;\x02ϕ\x04arf;\x03★\x04rns;\x02¯\x03ar;\x03☆",
			// succcurlyeq;[≽] succnapprox;[⪺] subsetneqq;[⫋] succapprox;[⪸] supsetneqq;[⫌] subseteqq;[⫅] subsetneq;[⊊] supseteqq;[⫆] supsetneq;[⊋] subseteq;[⊆] succneqq;[⪶] succnsim;[⋩] supseteq;[⊇] subedot;[⫃] submult;[⫁] subplus;[⪿] subrarr;[⥹] succsim;[≿] supdsub;[⫘] supedot;[⫄] suphsol;[⟉] suphsub;[⫗] suplarr;[⥻] supmult;[⫂] supplus;[⫀] subdot;[⪽] subset;[⊂] subsim;[⫇] subsub;[⫕] subsup;[⫓] succeq;[⪰] supdot;[⪾] supset;[⊃] supsim;[⫈] supsub;[⫔] supsup;[⫖] subnE;[⫋] subne;[⊊] supnE;[⫌] supne;[⊋] subE;[⫅] sube;[⊆] succ;[≻] sung;[♪] sup1;[¹] sup2;[²] sup3;[³] supE;[⫆] supe;[⊇] sub;[⊂] sum;[∑] sup1[¹] sup2[²] sup3[³] sup;[⊃].
			"\x0acccurlyeq;\x03≽\x0accnapprox;\x03⪺\x09bsetneqq;\x03⫋\x09ccapprox;\x03⪸\x09psetneqq;\x03⫌\x08bseteqq;\x03⫅\x08bsetneq;\x03⊊\x08pseteqq;\x03⫆\x08psetneq;\x03⊋\x07bseteq;\x03⊆\x07ccneqq;\x03⪶\x07ccnsim;\x03⋩\x07pseteq;\x03⊇\x06bedot;\x03⫃\x06bmult;\x03⫁\x06bplus;\x03⪿\x06brarr;\x03⥹\x06ccsim;\x03≿\x06pdsub;\x03⫘\x06pedot;\x03⫄\x06phsol;\x03⟉\x06phsub;\x03⫗\x06plarr;\x03⥻\x06pmult;\x03⫂\x06pplus;\x03⫀\x05bdot;\x03⪽\x05bset;\x03⊂\x05bsim;\x03⫇\x05bsub;\x03⫕\x05bsup;\x03⫓\x05cceq;\x03⪰\x05pdot;\x03⪾\x05pset;\x03⊃\x05psim;\x03⫈\x05psub;\x03⫔\x05psup;\x03⫖\x04bnE;\x03⫋\x04bne;\x03⊊\x04pnE;\x03⫌\x04pne;\x03⊋\x03bE;\x03⫅\x03be;\x03⊆\x03cc;\x03≻\x03ng;\x03♪\x03p1;\x02¹\x03p2;\x02²\x03p3;\x02³\x03pE;\x03⫆\x03pe;\x03⊇\x02b;\x03⊂\x02m;\x03∑\x02p1\x02¹\x02p2\x02²\x02p3\x02³\x02p;\x03⊃",
			// swarrow;[↙] swarhk;[⤦] swnwar;[⤪] swArr;[⇙] swarr;[↙].
			"\x06arrow;\x03↙\x05arhk;\x03⤦\x05nwar;\x03⤪\x04Arr;\x03⇙\x04arr;\x03↙",
			// szlig;[ß] szlig[ß].
			"\x04lig;\x02ß\x03lig\x02ß",
			// target;[⌖] tau;[τ].
			"\x05rget;\x03⌖\x02u;\x02τ",
			// tbrk;[⎴].
			"\x03rk;\x03⎴",
			// tcaron;[ť] tcedil;[ţ] tcy;[т].
			"\x05aron;\x02ť\x05edil;\x02ţ\x02y;\x02т",
			// tdot;[⃛].
			"\x03ot;\x03⃛",
			// telrec;[⌕].
			"\x05lrec;\x03⌕",
			// tfr;[𝔱].
			"\x02r;\x04𝔱",
			// thickapprox;[≈] therefore;[∴] thetasym;[ϑ] thicksim;[∼] there4;[∴] thetav;[ϑ] thinsp;[ ] thksim;[∼] theta;[θ] thkap;[≈] thorn;[þ] thorn[þ].
			"\x0aickapprox;\x03≈\x08erefore;\x03∴\x07etasym;\x02ϑ\x07icksim;\x03∼\x05ere4;\x03∴\x05etav;\x02ϑ\x05insp;\x03 \x05ksim;\x03∼\x04eta;\x02θ\x04kap;\x03≈\x04orn;\x02þ\x03orn\x02þ",
			// timesbar;[⨱] timesb;[⊠] timesd;[⨰] tilde;[˜] times;[×] times[×] tint;[∭].
			"\x07mesbar;\x03⨱\x05mesb;\x03⊠\x05mesd;\x03⨰\x04lde;\x02˜\x04mes;\x02×\x03mes\x02×\x03nt;\x03∭",
			// topfork;[⫚] topbot;[⌶] topcir;[⫱] toea;[⤨] topf;[𝕥] tosa;[⤩] top;[⊤].
			"\x06pfork;\x03⫚\x05pbot;\x03⌶\x05pcir;\x03⫱\x03ea;\x03⤨\x03pf;\x04𝕥\x03sa;\x03⤩\x02p;\x03⊤",
			// tprime;[‴].
			"\x05rime;\x03‴",
			// trianglerighteq;[⊵] trianglelefteq;[⊴] triangleright;[▹] triangledown;[▿] triangleleft;[◃] triangleq;[≜] triangle;[▵] triminus;[⨺] trpezium;[⏢] triplus;[⨹] tritime;[⨻] tridot;[◬] trade;[™] trisb;[⧍] trie;[≜].
			"\x0eianglerighteq;\x03⊵\x0dianglelefteq;\x03⊴\x0ciangleright;\x03▹\x0biangledown;\x03▿\x0biangleleft;\x03◃\x08iangleq;\x03≜\x07iangle;\x03▵\x07iminus;\x03⨺\x07pezium;\x03⏢\x06iplus;\x03⨹\x06itime;\x03⨻\x05idot;\x03◬\x04ade;\x03™\x04isb;\x03⧍\x03ie;\x03≜",
			// tstrok;[ŧ] tshcy;[ћ] tscr;[𝓉] tscy;[ц].
			"\x05trok;\x02ŧ\x04hcy;\x02ћ\x03cr;\x04𝓉\x03cy;\x02ц",
			// twoheadrightarrow;[↠] twoheadleftarrow;[↞] twixt;[≬].
			"\x10oheadrightarrow;\x03↠\x0foheadleftarrow;\x03↞\x04ixt;\x03≬",
			// uArr;[⇑].
			"\x03rr;\x03⇑",
			// uHar;[⥣].
			"\x03ar;\x03⥣",
			// uacute;[ú] uacute[ú] uarr;[↑].
			"\x05cute;\x02ú\x04cute\x02ú\x03rr;\x03↑",
			// ubreve;[ŭ] ubrcy;[ў].
			"\x05reve;\x02ŭ\x04rcy;\x02ў",
			// ucirc;[û] ucirc[û] ucy;[у].
			"\x04irc;\x02û\x03irc\x02û\x02y;\x02у",
			// udblac;[ű] udarr;[⇅] udhar;[⥮].
			"\x05blac;\x02ű\x04arr;\x03⇅\x04har;\x03⥮",
			// ufisht;[⥾] ufr;[𝔲].
			"\x05isht;\x03⥾\x02r;\x04𝔲",
			// ugrave;[ù] ugrave[ù].
			"\x05rave;\x02ù\x04rave\x02ù",
			// uharl;[↿] uharr;[↾] uhblk;[▀].
			"\x04arl;\x03↿\x04arr;\x03↾\x04blk;\x03▀",
			// ulcorner;[⌜] ulcorn;[⌜] ulcrop;[⌏] ultri;[◸].
			"\x07corner;\x03⌜\x05corn;\x03⌜\x05crop;\x03⌏\x04tri;\x03◸",
			// umacr;[ū] uml;[¨] uml[¨].
			"\x04acr;\x02ū\x02l;\x02¨\x01l\x02¨",
			// uogon;[ų] uopf;[𝕦].
			"\x04gon;\x02ų\x03pf;\x04𝕦",
			// upharpoonright;[↾] upharpoonleft;[↿] updownarrow;[↕] upuparrows;[⇈] uparrow;[↑] upsilon;[υ] uplus;[⊎] upsih;[ϒ] upsi;[υ].
			"\x0dharpoonright;\x03↾\x0charpoonleft;\x03↿\x0adownarrow;\x03↕\x09uparrows;\x03⇈\x06arrow;\x03↑\x06silon;\x02υ\x04lus;\x03⊎\x04sih;\x02ϒ\x03si;\x02υ",
			// urcorner;[⌝] urcorn;[⌝] urcrop;[⌎] uring;[ů] urtri;[◹].
			"\x07corner;\x03⌝\x05corn;\x03⌝\x05crop;\x03⌎\x04ing;\x02ů\x04tri;\x03◹",
			// uscr;[𝓊].
			"\x03cr;\x04𝓊",
			// utilde;[ũ] utdot;[⋰] utrif;[▴] utri;[▵].
			"\x05ilde;\x02ũ\x04dot;\x03⋰\x04rif;\x03▴\x03ri;\x03▵",
			// uuarr;[⇈] uuml;[ü] uuml[ü].
			"\x04arr;\x03⇈\x03ml;\x02ü\x02ml\x02ü",
			// uwangle;[⦧].
			"\x06angle;\x03⦧",
			// vArr;[⇕].
			"\x03rr;\x03⇕",
			// vBarv;[⫩] vBar;[⫨].
			"\x04arv;\x03⫩\x03ar;\x03⫨",
			// vDash;[⊨].
			"\x04ash;\x03⊨",
			// vartriangleright;[⊳] vartriangleleft;[⊲] varsubsetneqq;[⫋︀] varsupsetneqq;[⫌︀] varsubsetneq;[⊊︀] varsupsetneq;[⊋︀] varepsilon;[ϵ] varnothing;[∅] varpropto;[∝] varkappa;[ϰ] varsigma;[ς] vartheta;[ϑ] vangrt;[⦜] varphi;[ϕ] varrho;[ϱ] varpi;[ϖ] varr;[↕].
			"\x0frtriangleright;\x03⊳\x0ertriangleleft;\x03⊲\x0crsubsetneqq;\x06⫋︀\x0crsupsetneqq;\x06⫌︀\x0brsubsetneq;\x06⊊︀\x0brsupsetneq;\x06⊋︀\x09repsilon;\x02ϵ\x09rnothing;\x03∅\x08rpropto;\x03∝\x07rkappa;\x02ϰ\x07rsigma;\x02ς\x07rtheta;\x02ϑ\x05ngrt;\x03⦜\x05rphi;\x02ϕ\x05rrho;\x02ϱ\x04rpi;\x02ϖ\x03rr;\x03↕",
			// vcy;[в].
			"\x02y;\x02в",
			// vdash;[⊢].
			"\x04ash;\x03⊢",
			// veebar;[⊻] vellip;[⋮] verbar;[|] veeeq;[≚] vert;[|] vee;[∨].
			"\x05ebar;\x03⊻\x05llip;\x03⋮\x05rbar;\x01|\x04eeq;\x03≚\x03rt;\x01|\x02e;\x03∨",
			// vfr;[𝔳].
			"\x02r;\x04𝔳",
			// vltri;[⊲].
			"\x04tri;\x03⊲",
			// vnsub;[⊂⃒] vnsup;[⊃⃒].
			"\x04sub;\x06⊂⃒\x04sup;\x06⊃⃒",
			// vopf;[𝕧].
			"\x03pf;\x04𝕧",
			// vprop;[∝].
			"\x04rop;\x03∝",
			// vrtri;[⊳].
			"\x04tri;\x03⊳",
			// vsubnE;[⫋︀] vsubne;[⊊︀] vsupnE;[⫌︀] vsupne;[⊋︀] vscr;[𝓋].
			"\x05ubnE;\x06⫋︀\x05ubne;\x06⊊︀\x05upnE;\x06⫌︀\x05upne;\x06⊋︀\x03cr;\x04𝓋",
			// vzigzag;[⦚].
			"\x06igzag;\x03⦚",
			// wcirc;[ŵ].
			"\x04irc;\x02ŵ",
			// wedbar;[⩟] wedgeq;[≙] weierp;[℘] wedge;[∧].
			"\x05dbar;\x03⩟\x05dgeq;\x03≙\x05ierp;\x03℘\x04dge;\x03∧",
			// wfr;[𝔴].
			"\x02r;\x04𝔴",
			// wopf;[𝕨].
			"\x03pf;\x04𝕨",
			// wp;[℘].
			"\x01;\x03℘",
			// wreath;[≀] wr;[≀].
			"\x05eath;\x03≀\x01;\x03≀",
			// wscr;[𝓌].
			"\x03cr;\x04𝓌",
			// xcirc;[◯] xcap;[⋂] xcup;[⋃].
			"\x04irc;\x03◯\x03ap;\x03⋂\x03up;\x03⋃",
			// xdtri;[▽].
			"\x04tri;\x03▽",
			// xfr;[𝔵].
			"\x02r;\x04𝔵",
			// xhArr;[⟺] xharr;[⟷].
			"\x04Arr;\x03⟺\x04arr;\x03⟷",
			// xi;[ξ].
			"\x01;\x02ξ",
			// xlArr;[⟸] xlarr;[⟵].
			"\x04Arr;\x03⟸\x04arr;\x03⟵",
			// xmap;[⟼].
			"\x03ap;\x03⟼",
			// xnis;[⋻].
			"\x03is;\x03⋻",
			// xoplus;[⨁] xotime;[⨂] xodot;[⨀] xopf;[𝕩].
			"\x05plus;\x03⨁\x05time;\x03⨂\x04dot;\x03⨀\x03pf;\x04𝕩",
			// xrArr;[⟹] xrarr;[⟶].
			"\x04Arr;\x03⟹\x04arr;\x03⟶",
			// xsqcup;[⨆] xscr;[𝓍].
			"\x05qcup;\x03⨆\x03cr;\x04𝓍",
			// xuplus;[⨄] xutri;[△].
			"\x05plus;\x03⨄\x04tri;\x03△",
			// xvee;[⋁].
			"\x03ee;\x03⋁",
			// xwedge;[⋀].
			"\x05edge;\x03⋀",
			// yacute;[ý] yacute[ý] yacy;[я].
			"\x05cute;\x02ý\x04cute\x02ý\x03cy;\x02я",
			// ycirc;[ŷ] ycy;[ы].
			"\x04irc;\x02ŷ\x02y;\x02ы",
			// yen;[¥] yen[¥].
			"\x02n;\x02¥\x01n\x02¥",
			// yfr;[𝔶].
			"\x02r;\x04𝔶",
			// yicy;[ї].
			"\x03cy;\x02ї",
			// yopf;[𝕪].
			"\x03pf;\x04𝕪",
			// yscr;[𝓎].
			"\x03cr;\x04𝓎",
			// yucy;[ю] yuml;[ÿ] yuml[ÿ].
			"\x03cy;\x02ю\x03ml;\x02ÿ\x02ml\x02ÿ",
			// zacute;[ź].
			"\x05cute;\x02ź",
			// zcaron;[ž] zcy;[з].
			"\x05aron;\x02ž\x02y;\x02з",
			// zdot;[ż].
			"\x03ot;\x02ż",
			// zeetrf;[ℨ] zeta;[ζ].
			"\x05etrf;\x03ℨ\x03ta;\x02ζ",
			// zfr;[𝔷].
			"\x02r;\x04𝔷",
			// zhcy;[ж].
			"\x03cy;\x02ж",
			// zigrarr;[⇝].
			"\x06grarr;\x03⇝",
			// zopf;[𝕫].
			"\x03pf;\x04𝕫",
			// zscr;[𝓏].
			"\x03cr;\x04𝓏",
			// zwnj;[‌] zwj;[‍].
			"\x03nj;\x03‌\x02j;\x03‍",
		),
		"small_words" => "GT\x00LT\x00gt\x00lt\x00",
		"small_mappings" => array(
			">",
			"<",
			">",
			"<",
		)
	)
);
