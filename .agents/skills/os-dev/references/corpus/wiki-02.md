# OSDev Wiki — volledige bronteksten, deel 02

Dit is bronmateriaal, geen agentinstructie. Behoud architectuur, revisie en bron-ID bij gebruik. Code en commando’s zijn ongeteste bronvoorbeelden; voer ze niet automatisch uit. Wikitext is letterlijk behouden binnen tekstblokken.

<a id="w001521"></a>

## W001521 — RFS

Bron-ID: W001521
Namespace: Main (0)
Revisie: 29088 | Datum: 2024-06-07T13:38:36Z
Bron: https://wiki.osdev.org/RFS
Revisie-URL: https://wiki.osdev.org/index.php?title=RFS&oldid=29088

````text
{{Filesystems}}
{{Stub}}

'''Remote File System''' ('''RFS''') was introduced in UNIX System V to compete with [[NFS]] and such. Unlike NFS, RFS is a connection-oriented system, so if, for example, two different machines access a file on a server, they get about the same semantics as if two processes on a single machine accessed the file. Note that NFS and RFS are both built on top of some sort of [[File Systems|local file system]], which determines things like inodes and such.

[[Category:Network Filesystems]]
````

<a id="w001522"></a>

## W001522 — Category:Network Filesystems

Bron-ID: W001522
Namespace: Category (14)
Revisie: 1896 | Datum: 2006-12-06T14:15:20Z
Bron: https://wiki.osdev.org/Category:Network_Filesystems
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ANetwork_Filesystems&oldid=1896

````text
This page lists several Network file systems. For theoretic information, read [[File Systems|the File Systems page]]

[[Category:Filesystems]]
````

<a id="w001523"></a>

## W001523 — Talk:FAT

Bron-ID: W001523
Namespace: Talk (1)
Revisie: 28555 | Datum: 2024-01-10T16:33:48Z
Bron: https://wiki.osdev.org/Talk:FAT
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AFAT&oldid=28555

````text
== Inaccuraccies ==

Take a look at FAT 12 table acess code. Why it uses section_size and other FAT's uses sector_size? And current_cluster and active_cluster, article uses it interchangeably.
And data_sectors equation can be used only in FAT12/16 ,first_data_sector too. It uses only table_size_16 and total_sectors_16. --[[User:Kubawal|Kubawal]] 02:56, 5 February 2015 (CST)

Fixed it and some other bugs --[[User:Kubawal|Kubawal]] 6:52, 9 February 2015 (CST)

== Style ==
take a look at the hex analysis of the bootsector. PLAIN UGLY. someone know how to fix that without breaking the monospaced font? - [[User:Combuster|Combuster]] 09:00, 6 December 2006 (CST)

How's that? -[[User:Jhawthorn|Jhawthorn]] 00:33, 7 December 2006 (CST)

Nice work - [[User:Combuster|Combuster]] 04:41, 7 December 2006 (CST)


The section "File Allocation Table Entry Cluster Values" Is plainly rediculous... It seems like almost a joke on the reader? I mean stuff like 

"AND the binary value of the hexadecimal number of 8007 (1000 0000 0000 0111) to the binary value of the hexadecimal number 0FFF (0000 1111 1111 1111). The result is **7 (hex or decimal). "

It goes back and forth about hex vs decimal vs binary.... A number is a number, no matter what base it's in... the whole section just seems like it's made to be as confusing as possible... or is it just me? I'm not trying to be critical but it does seem to be done that way on purpose. I'll try to rewrite that section if I can obtain the correct knowledge somewhere.  ---Posted by Dkelly.

Please sign your posts Dkelly! --[[User:Brynet-Inc|Brynet-Inc]] 07:10, 10 November 2007 (CST)


I've cleaned it up a lot. No more crazy hex->decimal or decimal->hex conversions. Nicer flow etc. If you miss something from the old page be sure to mention it (or add it back in yourself :) ). 
[[User:Stevo14|Stevo14]] 09:21, 5 July 2008 (UTC)


Calculations regarding the size of the root directory was completely missing. I added it in because the calculation were important for FAT12/16. Also, I could not find anything that talks about "relative" and "absolute" clusters in the hardware white paper (version 1.03 December 6, 2000, Microsoft). -7 July 2008 by uglyoldbob

By relative and absolute I meant cluster numbers measured from the beginning of the free space or the beginning of the volume. AFAIK, all of the cluster numbers that you read out of the directories are relative to the start of the free space, not the start of the volume. [[User:Stevo14|Stevo14]] 12:03, 7 July 2008 (UTC)


There seems to be some contradiction concerning the extended boot record for fat32:
Offset/Length/Description
{| {{wikitable}}
|-
! Offset 
! Length 
! Meaning
|-
| 36      
| 4
|	The size of the File Allocation Table in bytes.
|- 
| 40      
| 2
|	Flags.
|- 
| 42
| 1
|	Signature (must be 0x28 or 0x29).
|- 
| 39 
| 2
|       FAT version number.
|-
|}
Am I right in assuming that the signature needs to be removed (it exists twice in the table) and the offset for the fat version number need to be changed to 42?
Furthermore, the fat size is written as being in bytes, while wikipedia claims that offset 0x24/36d ist "Sectors per file allocation table".
Does someone know which is right?
[[User:Narida|Narida]] 15:16, 11 August 2009 (UTC)


----
A certain document from a certain source tells me that at offset 40 is an 'extended flags' field of 2 bytes, followed by a 'filesystem version' field of 2 bytes. So, I believe you are correct in that the signature field should be removed there, and that the offset of the 'filesystem version' field should be changed to 42.

[[User:Bughunter|bughunter2]] 20:50, 11 August 2009 (UTC)

== exFAT ==

I'm not sure, but it might be interesting to add information regarding exFAT to the topic. Since it's Microsoft's "replacement for FAT" after all. I don't know too much about all these file systems, so I'm just dropping in idea's here. --[[User:Creature|Creature]] 10:43, 21 December 2009 (UTC)

== Long file names ==
US patent 5579517 A expired in 2015 therefore I have removed references to LFN being patented [[User:Jnc100|John]] 16:45, 18 November 2016 (CST)

== Adding information ==

If the disk is fairly small the root_dir can be at the end of the disk.
Shouldn't it be precised when getting root dir ?
````

<a id="w001525"></a>

## W001525 — Category talk:PowerManagement

Bron-ID: W001525
Namespace: Category talk (15)
Revisie: 1903 | Datum: 2006-12-06T23:54:12Z
Bron: https://wiki.osdev.org/Category_talk:PowerManagement
Revisie-URL: https://wiki.osdev.org/index.php?title=Category_talk%3APowerManagement&oldid=1903

````text
Would it be better to post the useful code instead of telling someone to read some OS' sources - [[User:Combuster|Combuster]] 17:54, 6 December 2006 (CST)
````

<a id="w001526"></a>

## W001526 — Category:Stubs

Bron-ID: W001526
Namespace: Category (14)
Revisie: 2349 | Datum: 2006-12-16T21:06:09Z
Bron: https://wiki.osdev.org/Category:Stubs
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AStubs&oldid=2349

````text
Lists all articles using the [[Template:Stub|stub]] template.

[[Category:Wiki administration]]
````

<a id="w001527"></a>

## W001527 — OSDev Wiki:Pages To Convert/Questionable pages

Bron-ID: W001527
Namespace: OSDev Wiki (4)
Revisie: 3239 | Datum: 2007-03-08T16:20:00Z
Bron: https://wiki.osdev.org/OSDev_Wiki:Pages_To_Convert/Questionable_pages
Revisie-URL: https://wiki.osdev.org/index.php?title=OSDev_Wiki%3APages_To_Convert%2FQuestionable_pages&oldid=3239

````text
== Questionable pages ==

===User Pages===
These are pages relating to people. Some of them should be converted. Others should be left for the person named in the page to recreate at "User:username"

*[[mtfaq:ChrisGiese|ChrisGiese]]<br />
*[[mtfaq:AlexeiFrounze|AlexeiFrounze]]<br />
*[[mtfaq:bubach|bubach]]<br />
*[[mtfaq:dembol|dembol]]<br />
*[[mtfaq:Xardfir|Xardfir]]<br />
*[[mtfaq:StephaneTravostino|StephaneTravostino]]<br />
*[[mtfaq:SteveWainstead|SteveWainstead]]<br />
*[[mtfaq:SterphenBall|SterphenBall]]<br />
*[[mtfaq:TheNeuromancer|TheNeuromancer]]<br />
*[[mtfaq:StuGeorge|StuGeorge]]<br />
*[[mtfaq:Pete|Pete]]<br />
*[[mtfaq:PeterMoore|PeterMoore]]<br />
*[[mtfaq:ZomgTest|ZomgTest]]<br />
*[[mtfaq:NotTheCHEAT|NotTheCHEAT]]<br />
*[[mtfaq:MrSmiddy|MrSmiddy]]<br />
*[[mtfaq:LionelThiry|LionelThiry]]<br />
*[[mtfaq:MarcusWarga|MarcusWarga]]<br />
*[[mtfaq:JosephOsako|JosephOsako]]<br />
*[[mtfaq:LarsOlson|LarsOlson]]<br />
*[[mtfaq:MirAbile|MirAbile]]<br />
*[[mtfaq:TheKemp|TheKemp]]<br />
*[[mtfaq:EliGottlieb|EliGottlieb]]<br />
*[[mtfaq:ErikGreenwald|ErikGreenwald]]<br />
*[[mtfaq:Keeper|Keeper]]<br />
*[[mtfaq:DegenerateOS|DegenerateOS]]<br />
*[[mtfaq:DasCandy|DasCandy]]<br />
*[[mtfaq:CjMovie|CjMovie]]<br />
*[[mtfaq:BrandonKilgore|BrandonKilgore]]<br />
*[[mtfaq:BrendanTrotter|BrendanTrotter]]<br />
*[[mtfaq:Bluecode|Bluecode]]<br />
*[[mtfaq:BeyondInfinity|BeyondInfinity]]<br />
*[[mtfaq:Anton|Anton]]<br />
*[[mtfaq:PypeClicker|PypeClicker]]<br />
*[[mtfaq:ArtoBendiken|ArtoBendiken]]<br />
*[[mtfaq:TimRobinson|TimRobinson]]<br />
*[[mtfaq:MartinBaute|MartinBaute]]<br />
*[[mtfaq:DennisCgc|DennisCgc]]<br />
*[[mtfaq:AaronLiske|AaronLiske]]<br />
*[[mtfaq:BruceJohnston|BruceJohnston]]<br />
*[http://www.osdev.org/osfaq2/index.php/ReiniUrban ReiniUrban]<br />
*[http://www.osdev.org/osfaq2/index.php/DaidalosGuy DaidalosGuy]<br />
*[http://www.osdev.org/osfaq2/index.php/MystranTheGreen MystranTheGreen]<br />

=== All in ones ===
These pages are big lists of includes:
*[http://www.osdev.org/osfaq2/index.php/NasmAllInOne NasmAllInOne]<br />
*[http://www.osdev.org/osfaq2/index.php/GasAllInOne GasAllInOne]<br />
*[http://www.osdev.org/osfaq2/index.php/AsmExample AsmExample]<br />
*[http://www.osdev.org/osfaq2/index.php/DesignAllInOne DesignAllInOne]<br />
*[http://www.osdev.org/osfaq2/index.php/all%20in%20one all in one]<br />
*[http://www.osdev.org/osfaq2/index.php/ReferenceAllInOne ReferenceAllInOne]<br />

=== Stubs ===
These pages are the stubs from Megatokyo... they have no useful data as far as I can tell...
*[[mtfaq:Why cant I access all my memory?|Why cant I access all my memory?]]
*[[mtfaq:Where can I find programming info on PCI?|Where can I find programming info on PCI?]]
*[[mtfaq:I heard you can do PCI calls with the BIOS in Protected Mode?|I heard you can do PCI calls with the BIOS in Protected Mode?]]

=== Metapages ===
These are miscellaneous admin pages, relating to the functioning of PHPWiki; they probably don't need to be moved.

Somebody should double check, though...
[[mtfaq:UnfoldSubpagesPlugin|UnfoldSubpagesPlugin]]<br />
[[mtfaq:WikiBlogPlugin|WikiBlogPlugin]]<br />
[[mtfaq:TeX2pngPlugin|TeX2pngPlugin]]<br />
[[mtfaq:TemplateExample|TemplateExample]]<br />
[[mtfaq:SystemInfoPlugin|SystemInfoPlugin]]<br />
[[mtfaq:RichTablePlugin|RichTablePlugin]]<br />
[[mtfaq:SearchHighlightPlugin|SearchHighlightPlugin]]<br />
[[mtfaq:RelatedChanges|RelatedChanges]]<br />
[[mtfaq:ReleaseNotes|ReleaseNotes]]<br />
[[mtfaq:WikiAdminSelect|WikiAdminSelect]]<br />
[[mtfaq:RssFeedPlugin|RssFeedPlugin]]<br />
[[mtfaq:TranslateText|TranslateText]]<br />
[[mtfaq:UpLoadPlugin|UpLoadPlugin]]<br />
[[mtfaq:TemplatePlugin|TemplatePlugin]]<br />
[[mtfaq:RandomPage|RandomPage]]<br />
[[mtfaq:RawHtmlPlugin|RawHtmlPlugin]]<br />
[[mtfaq:RecentComments|RecentComments]]<br />
[[mtfaq:PhotoAlbumPlugin|PhotoAlbumPlugin]]<br />
[[mtfaq:PhpHighlightPlugin|PhpHighlightPlugin]]<br />
[[mtfaq:SyntaxHighlighterPlugin|SyntaxHighlighterPlugin]]<br />
[[mtfaq:PhpWeatherPlugin|PhpWeatherPlugin]]<br />
[[mtfaq:PhpWikiAdministration/Remove|PhpWikiAdministration/Remove]]<br />
[[mtfaq:PhpWikiAdministration/Rename|PhpWikiAdministration/Rename]]<br />
[[mtfaq:PhpWikiAdministration/Replace|PhpWikiAdministration/Replace]]<br />
[[mtfaq:PhpWikiAdministration/SetAcl|PhpWikiAdministration/SetAcl]]<br />
[[mtfaq:PhpWikiAdministration/Chmod|PhpWikiAdministration/Chmod]]<br />
[[mtfaq:PhpWikiAdministration/Chown|PhpWikiAdministration/Chown]]<br />
[[mtfaq:PhpWikiPoll|PhpWikiPoll]]<br />
[[mtfaq:PhpWikiRecentChanges|PhpWikiRecentChanges]]<br />
[[mtfaq:PloticusPlugin|PloticusPlugin]]<br />
[[mtfaq:TranscludePlugin|TranscludePlugin]]<br />
[[mtfaq:WhoIsOnline|WhoIsOnline]]<br />
[[mtfaq:AddCommentPlugin|AddCommentPlugin]]<br />
[[mtfaq:WikiPlugin|WikiPlugin]]<br />
[[mtfaq:PhpWikiDocumentation|PhpWikiDocumentation]]<br />
[[mtfaq:AddingPages|AddingPages]]<br />
[[mtfaq:TextFormattingRules|TextFormattingRules]]<br />
[[mtfaq:AllPages|AllPages]]<br />
[[mtfaq:OrphanedPages|OrphanedPages]]<br />
[[mtfaq:AllPagesCreatedByMe|AllPagesCreatedByMe]]<br />
[[mtfaq:AllPagesLastEditedByMe|AllPagesLastEditedByMe]]<br />
[[mtfaq:AllPagesOwnedByMe|AllPagesOwnedByMe]]<br />
[[mtfaq:AllUsers|AllUsers]]<br />
[[mtfaq:UserPreferences|UserPreferences]]<br />
[[mtfaq:PluginManager|PluginManager]]<br />
[[mtfaq:AnalyseAccessLogSqlPlugin|AnalyseAccessLogSqlPlugin]]<br />
[[mtfaq:AppendText|AppendText]]<br />
[[mtfaq:AppendTextPlugin|AppendTextPlugin]]<br />
[[mtfaq:AuthorHistoryPlugin|AuthorHistoryPlugin]]<br />
[[mtfaq:BackLinks|BackLinks]]<br />
[[mtfaq:CalendarListPlugin|CalendarListPlugin]]<br />
[[mtfaq:CalendarPlugin|CalendarPlugin]]<br />
[[mtfaq:CommentPlugin|CommentPlugin]]<br />
[[mtfaq:EditMetaData|EditMetaData]]<br />
[[mtfaq:EditMetaDataPlugin|EditMetaDataPlugin]]<br />
[[mtfaq:EditText|EditText]]<br />
[[mtfaq:InterWiki|InterWiki]]<br />
[[mtfaq:InterWikiMap|InterWikiMap]]<br />
[[mtfaq:FindPage|FindPage]]<br />
[[mtfaq:HowToUseWiki|HowToUseWiki]]<br />
[[mtfaq:RecentChanges|RecentChanges]]<br />
[[mtfaq:FrameIncludePlugin|FrameIncludePlugin]]<br />
[[mtfaq:IncludePagePlugin|IncludePagePlugin]]<br />
[[mtfaq:FullRecentChanges|FullRecentChanges]]<br />
[[mtfaq:RecentEdits|RecentEdits]]<br />
[[mtfaq:FullTextSearch|FullTextSearch]]<br />
[[mtfaq:FuzzyPages|FuzzyPages]]<br />
[[mtfaq:HelloWorldPlugin|HelloWorldPlugin]]<br />
[[mtfaq:WikiWikiWeb|WikiWikiWeb]]<br />
[[mtfaq:SandBox|SandBox]]<br />
[[mtfaq:WikiIssues|WikiIssues]]<br />
[[mtfaq:RecentVisitors|RecentVisitors]]<br />
[[mtfaq:MostPopular|MostPopular]]<br />
[[mtfaq:PhpWikiAdministration|PhpWikiAdministration]]<br />
[[mtfaq:HomePageAlias|HomePageAlias]]<br />
[[mtfaq:RedirectToPlugin|RedirectToPlugin]]<br />
[[mtfaq:OldTextFormattingRules|OldTextFormattingRules]]<br />
[[mtfaq:InterWikiSearch|InterWikiSearch]]<br />
[[mtfaq:LikePages|LikePages]]<br />
[[mtfaq:LinkDatabase|LinkDatabase]]<br />
[[mtfaq:LinkIcons|LinkIcons]]<br />
[[mtfaq:ListPagesPlugin|ListPagesPlugin]]<br />
[[mtfaq:MagicPhpWikiURLs|MagicPhpWikiURLs]]<br />
[[mtfaq:TitleSearch|TitleSearch]]<br />
[[mtfaq:ModeratedPage|ModeratedPage]]<br />
[[mtfaq:NewMarkupTestPage|NewMarkupTestPage]]<br />
[[mtfaq:OldMarkupTestPage|OldMarkupTestPage]]<br />
[[mtfaq:OldStyleTablePlugin|OldStyleTablePlugin]]<br />
[[mtfaq:PageGroupTest|PageGroupTest]]<br />
[[mtfaq:PageGroupTestOne|PageGroupTestOne]]<br />
[[mtfaq:PageGroupTestTwo|PageGroupTestTwo]]<br />
[[mtfaq:PageGroupTestThree|PageGroupTestThree]]<br />
[[mtfaq:PageGroupTestFour|PageGroupTestFour]]<br />
[[mtfaq:PageHistory|PageHistory]]<br />
[[mtfaq:PageInfo|PageInfo]]<br />
[[mtfaq:PagePermissions|PagePermissions]]<br />
[[mtfaq:PgsrcTranslation|PgsrcTranslation]]<br />
[[mtfaq:PgsrcTranslation/de|PgsrcTranslation/de]]<br />
[[mtfaq:PgsrcTranslation/es|PgsrcTranslation/es]]<br />
[[mtfaq:PgsrcTranslation/fr|PgsrcTranslation/fr]]<br />
[[mtfaq:PgsrcTranslation/it|PgsrcTranslation/it]]<br />
[[mtfaq:PgsrcTranslation/ja|PgsrcTranslation/ja]]<br />
[[mtfaq:PgsrcTranslation/nl|PgsrcTranslation/nl]]<br />
[[mtfaq:PgsrcTranslation/zh|PgsrcTranslation/zh]]<br />
[[mtfaq:PgsrcTranslation_sv|PgsrcTranslation_sv]]<br />
[http://www.osdev.org/osfaq2/index.php/ProjectSummary ProjectSummary]<br />
[http://www.osdev.org/osfaq2/index.php/RecentReleases RecentReleases]<br />
[http://www.osdev.org/osfaq2/index.php/_AuthInfo _AuthInfo]<br />
[http://www.osdev.org/osfaq2/index.php/_MailifyPage _MailifyPage]<br />
[http://www.osdev.org/osfaq2/index.php/UpLoad UpLoad]<br />
[http://www.osdev.org/osfaq2/index.php/WantedPages WantedPages]<br />
[http://www.osdev.org/osfaq2/index.php/PageDump PageDump]<br />
[http://www.osdev.org/osfaq2/index.php/RateIt RateIt]<br />
[http://www.osdev.org/osfaq2/index.php/GoodStyle GoodStyle]<br />
[http://www.osdev.org/osfaq2/index.php/GoogleLink GoogleLink]<br />
[http://www.osdev.org/osfaq2/index.php/MoreAboutMechanics MoreAboutMechanics]<br />
[http://www.osdev.org/osfaq2/index.php/FoafViewerPlugin FoafViewerPlugin]<br />
[http://www.osdev.org/osfaq2/index.php/DebugInfo DebugInfo]<br />
[http://www.osdev.org/osfaq2/index.php/PhpWiki PhpWiki]<br />
[http://www.osdev.org/osfaq2/index.php/HomePage HomePage]<br />
[http://www.osdev.org/osfaq2/index.php/CreatePage CreatePage]<br />
[http://www.osdev.org/osfaq2/index.php/CreatePagePlugin CreatePagePlugin]<br />
[http://www.osdev.org/osfaq2/index.php/CreateTocPlugin CreateTocPlugin]<br />
[http://www.osdev.org/osfaq2/index.php/ExternalSearchPlugin ExternalSearchPlugin]<br />
````

<a id="w001528"></a>

## W001528 — C++

Bron-ID: W001528
Namespace: Main (0)
Revisie: 30364 | Datum: 2026-03-22T16:20:56Z
Bron: https://wiki.osdev.org/C%2B%2B
Revisie-URL: https://wiki.osdev.org/index.php?title=C%2B%2B&oldid=30364

````text
:''For a quick tutorial on getting a bare bones C++ kernel running see [[Bare Bones]]''

A kernel can be programmed in C++, it is very similar to making a kernel in C, except that there are a few pitfalls you must take into account (runtime support, constructors, ...). This page will not list any (dis)advantages of this approach, but rather what you need to do to get things fired up.

A lot of features C++ offers can be used on-the-fly; they require no additional support or code to use them properly (e.g. templates, classes, inheritance, virtual functions). There are however other parts of C++ that do require runtime support, which will be discussed in this article.

== Introduction ==

If you have created a C++ kernel as documented in the [[Bare Bones]] article, then many C++ features are already available and work out the box. However, your kernel does not yet satisfy the ABI and you cannot be confident that the compiler will not emit problematic code, even if you stick to the intersection of C and C++. In particular, you may need to initialize further CPU state to enable the floating-point registers and instructions, as the compiler has every reason to think floating point registers and instructions are available by default.

However, the compiler will assume that all the C++ runtime support is available by default, however you are not linking in libsupc++ into your C++ kernel, which implements the necessary run-time support. This is why you are passing -fno-rtti and -fno-exceptions to your cross-compiler to let these runtime features are unavailable. Going further, you should link in libsupc++ into your kernel, but at the moment it's known to not be readily accessible to those starting out with operating systems development and the GCC build process doesn't cross-compile it properly for the bare -elf platforms by default.

You also need to call the global constructors as documented in [[Calling Global Constructors]] to satisfy the ABI requirement that the program initialization tasks are properly called.

== Pure virtual functions ==

If you want to use pure virtual functions, your compiler needs a single support function. It is only called in case a pure virtual function call cannot be made (e.g. if you have overridden the virtual function table of an object). But nonetheless your linker will complain about unresolved symbols, if you use pure virtual functions and don't provide this support routine.

Enabling pure virtual functions in GCC is fairly straightforward. All you need to do is add the function below to one of your C++ source files (and make sure it is linked in). It is not necessary to declare this function first, the definition alone is good enough for GCC. The function itself doesn't even need to do anything (and it doesn't in most implementations), it just needs to be "there" just in case.

Below you will find an example of an implementation in respectively GCC.

<syntaxhighlight lang="cpp">
extern "C" void __cxa_pure_virtual()
{
    // Do nothing or print an error message.
}
</syntaxhighlight>

Or, if you happen to use Visual Studio:

<syntaxhighlight lang="cpp">
int __cdecl _purecall()
{
    // Do nothing or print an error message.
}
</syntaxhighlight>

If, during runtime, your kernel detects that a call to a pure virtual function couldn't be made, it calls the above functions. These functions should actually never be called, because without hacks, or through undefined behaviour of your kernel, it is not possible to instantiate a class that doesn't define all pure virtual functions.

== Global objects ==

{{In_Progress}}

TODO: Please unify this information with the newer [[Calling_Global_Constructors]] article.

Global objects must have their constructors called before they are used. Usually, they are called by the start-up code (which you just disabled). So, in order to be able to use them, you have to write your own start-up code for them. All objects have a constructor and a destructor. When an executable is loaded into memory and the program jumps straight to the entry point, the constructors of global objects will not have been called. One solution is to do this manually. You could put this code first when your C++ entry point is called:

<syntaxhighlight lang="cpp">
	object1.object1();
	object2.object2();
	object3.object3();
	// ...
</syntaxhighlight>

Global or static objects have to be constructed by the environment before they are available to C++. Care should be taken if global/static objects need '''new''' and '''delete''' in their constructors. In this case it is best to construct these objects only after your kernel heap is ready for use (and you have access to dynamic memory allocation). Not doing so can cause an object to attempt to allocate memory via the non-working '''new''' operator. This also simplifies the storing of the destructor functions in '''__cxa_atexit''', because you don't have to use a static and fixed-size structure.

=== GCC ===

Note: This appears to be specific to the Itanium platform. For IA-32/x86/i386 and amd64/x86_64, please check out [[Calling_Global_Constructors]] instead.

According to the [http://refspecs.freestandards.org/LSB_3.1.0/LSB-Core-generic/LSB-Core-generic/baselib---cxa-atexit.html Itanium C++ Application Binary Interface] (which '''g++''' follows and VC++ does not) the function '''__cxa_atexit''' is used to register a destructor that should be called when a shared library needs to be unloaded. It should insert a function pointer with maximum 1 accompanying argument and the handle of the object or shared resource to be destroyed into a table.

In the example implementation of '''__cxa_atexit''', the '''__atexit_funcs[ATEXIT_MAX_FUNCS]''' array acts as the table. This is why the '''__cxa_atexit''' function is defined as:

<syntaxhighlight lang="cpp">
int __cxa_atexit(void (*destructor) (void *), void *arg, void *__dso_handle);
</syntaxhighlight>

So that the '''destructor''' function pointer is the handle for a destructor function and '''arg''' is the single argument it may take. Finally, '''__dso_handle''' is a handle for the DSO (Dynamic Shared Object).

So summarized, you are required to define two functions and one symbol in order to use global objects in your C++ files:

<syntaxhighlight lang="cpp">
void *__dso_handle;

int __cxa_atexit(void (*destructor) (void *), void *arg, void *dso);
void __cxa_finalize(void *f);
</syntaxhighlight>

After you have called the objects constructor GCC automatically calls the function

<syntaxhighlight lang="cpp">
int __cxa_atexit(void (*destructor) (void *), void *arg, void *dso);
</syntaxhighlight>

This function should save all three parameters and if successful return zero, on failure non-zero. When your kernel exits you should call '''__cxa_finalize(0)'''. According to the ABI specification, calling this with 0 as the parameter instead of the address of a function (to be called and removed from the list) causes ''all'' destructors in the list to be called and removed from the list.

Since you will be calling this function from your Assembly source right after your kernel exits, you could use the following code:

<syntaxhighlight lang="nasm">
; This is NASM source, mind you.
sub esp, 4
mov [esp], dword 0x0

call __cxa_finalize

add esp, 4
</syntaxhighlight>

The following is tested, working, fully commented source that gives a more detailed explanation than the source previously found here. It also highlights what improvements can be implemented and where they can be inserted. To use it, just include '''icxxabi.h''' in any '''one''' file of your C++ kernel source (preferably the file where your kernel's main statements begin). 

'''File: icxxabi.h'''
<syntaxhighlight lang="cpp">
#ifndef _ICXXABI_H
	#define _ICXXABI_H

	#define ATEXIT_MAX_FUNCS	128
	
	#ifdef __cplusplus
	extern "C" {
	#endif
	
typedef unsigned uarch_t;

struct atexit_func_entry_t
{
	/*
	* Each member is at least 4 bytes large. Such that each entry is 12bytes.
	* 128 * 12 = 1.5KB exact.
	**/
	void (*destructor_func)(void *);
	void *obj_ptr;
	void *dso_handle;
};

int __cxa_atexit(void (*f)(void *), void *objptr, void *dso);
void __cxa_finalize(void *f);

	#ifdef __cplusplus
	};
	#endif

#endif
</syntaxhighlight>


'''File: icxxabi.cpp'''
<syntaxhighlight lang="cpp">

#include "./icxxabi.h"

	#ifdef __cplusplus
	extern "C" {
	#endif

atexit_func_entry_t __atexit_funcs[ATEXIT_MAX_FUNCS];
uarch_t __atexit_func_count = 0;

void *__dso_handle = 0; //Attention! Optimally, you should remove the '= 0' part and define this in your asm script.

int __cxa_atexit(void (*f)(void *), void *objptr, void *dso)
{
	if (__atexit_func_count >= ATEXIT_MAX_FUNCS) {return -1;};
	__atexit_funcs[__atexit_func_count].destructor_func = f;
	__atexit_funcs[__atexit_func_count].obj_ptr = objptr;
	__atexit_funcs[__atexit_func_count].dso_handle = dso;
	__atexit_func_count++;
	return 0; /*I would prefer if functions returned 1 on success, but the ABI says...*/
};

void __cxa_finalize(void *f)
{
	uarch_t i = __atexit_func_count;
	if (!f)
	{
		/*
		* According to the Itanium C++ ABI, if __cxa_finalize is called without a
		* function ptr, then it means that we should destroy EVERYTHING MUAHAHAHA!!
		*
		* TODO:
		* Note well, however, that deleting a function from here that contains a __dso_handle
		* means that one link to a shared object file has been terminated. In other words,
		* We should monitor this list (optional, of course), since it tells us how many links to 
		* an object file exist at runtime in a particular application. This can be used to tell 
		* when a shared object is no longer in use. It is one of many methods, however.
		**/
		//You may insert a prinf() here to tell you whether or not the function gets called. Testing
		//is CRITICAL!
		while (i--)
		{
			if (__atexit_funcs[i].destructor_func)
			{
				/* ^^^ That if statement is a safeguard...
				* To make sure we don't call any entries that have already been called and unset at runtime.
				* Those will contain a value of 0, and calling a function with value 0
				* will cause undefined behaviour. Remember that linear address 0, 
				* in a non-virtual address space (physical) contains the IVT and BDA.
				*
				* In a virtual environment, the kernel will receive a page fault, and then probably
				* map in some trash, or a blank page, or something stupid like that.
				* This will result in the processor executing trash, and...we don't want that.
				**/
				(*__atexit_funcs[i].destructor_func)(__atexit_funcs[i].obj_ptr);
			};
		};
		return;
	};

	while (i--)
	{
		/*
		* The ABI states that multiple calls to the __cxa_finalize(destructor_func_ptr) function
		* should not destroy objects multiple times. Only one call is needed to eliminate multiple
		* entries with the same address.
		*
		* FIXME:
		* This presents the obvious problem: all destructors must be stored in the order they
		* were placed in the list. I.e: the last initialized object's destructor must be first
		* in the list of destructors to be called. But removing a destructor from the list at runtime
		* creates holes in the table with unfilled entries.
		* Remember that the insertion algorithm in __cxa_atexit simply inserts the next destructor
		* at the end of the table. So, we have holes with our current algorithm
		* This function should be modified to move all the destructors above the one currently
		* being called and removed one place down in the list, so as to cover up the hole.
		* Otherwise, whenever a destructor is called and removed, an entire space in the table is wasted.
		**/
		if (__atexit_funcs[i].destructor_func == f)
		{
			/* 
			* Note that in the next line, not every destructor function is a class destructor.
			* It is perfectly legal to register a non class destructor function as a simple cleanup
			* function to be called on program termination, in which case, it would not NEED an
			* object This pointer. A smart programmer may even take advantage of this and register
			* a C function in the table with the address of some structure containing data about
			* what to clean up on exit.
			* In the case of a function that takes no arguments, it will simply be ignore within the
			* function itself. No worries.
			**/
			(*__atexit_funcs[i].destructor_func)(__atexit_funcs[i].obj_ptr);
			__atexit_funcs[i].destructor_func = 0;
			
			/*
			* Notice that we didn't decrement __atexit_func_count: this is because this algorithm
			* requires patching to deal with the FIXME outlined above.
			**/
		};
	};
};

	#ifdef __cplusplus
	};
	#endif
</syntaxhighlight>

=== Visual C++ ===
{{main|Visual Studio}}
Running constructors and destructors is covered in MSDN help and in the C runtime library sources. See '''#pragma init_seg''' on MSDN for more information.

Basically what happens is that pointers to functions are placed in '''.CRT$XIC, $XIL, $XIU''' based on the value of '''init_seg'''. The linker then merges everything together in the '''.CRT''' section, in the order of the letters after the '''$'''. The pointers between the XIA ('''xi_a''') and XIZ ('''xi_z''') are then called if non-zero. The '''.CRT''' section is merged with the '''.data''' section to avoid a completely separated section.

One problem with C++ support is the horrible name-mangling that is impossible to read in the map file. A build script should be set up that runs the map file through the '''undname.exe''' tool, so that names like '''??2@YAPAXI@Z''' (operator new - I think...) and others are readable.

Below you will find some example code. Simply call '''runInit()''' if you want to initialize any static objects and then call '''runTerm()''' if static object destructors are to be run.

<syntaxhighlight lang="cpp">
typedef void (*_PVFV)(void);
typedef int  (*_PIFV)(void);
typedef void (*_PVFI)(int);

#pragma data_seg(".CRT$XIA")
__declspec(allocate(".CRT$XIA")) _PIFV __xi_a[] = {0};
#pragma data_seg(".CRT$XIZ")
__declspec(allocate(".CRT$XIZ")) _PIFV __xi_z[] = {0};
#pragma data_seg(".CRT$XCA")
__declspec(allocate(".CRT$XCA")) _PVFV __xc_a[] = {0};
#pragma data_seg(".CRT$XCZ")
__declspec(allocate(".CRT$XCZ")) _PVFV __xc_z[] = {0};
#pragma data_seg(".CRT$XPA")
__declspec(allocate(".CRT$XPA")) _PVFV __xp_a[] = {0};
#pragma data_seg(".CRT$XPZ")
__declspec(allocate(".CRT$XPZ")) _PVFV __xp_z[] = {0};
#pragma data_seg(".CRT$XTA")
__declspec(allocate(".CRT$XTA")) _PVFV __xt_a[] = {0};
#pragma data_seg(".CRT$XTZ")
__declspec(allocate(".CRT$XTZ")) _PVFV __xt_z[] = {0};
#pragma data_seg()
#pragma comment(linker, "/merge:.CRT=.data")

static _PVFV onexitarray[32];
static _PVFV *onexitbegin, *onexitend;

int __cdecl _purecall()
{
        // print error message
}

int __cdecl atexit(_PVFV fn)
{
        if (32*4 < ((int)onexitend-(int)onexitbegin)+4)
                return 1;
        else
                *(onexitend++) = fn;
        return 0;
}

EXTERN int runInit()
{
        // init the __xi_a to __xi_z:  __initex(__xi_a, __xi_z);
        // init __xc_a to __xc_z
}

static void __init(_PVFV *pfbegin, _PVFV *pfend)
{
    while (pfbegin < pfend)
    {
        if (*pfbegin != 0)
            (**pfbegin)();
        ++pfbegin;
    }
}

static int __initex(_PIFV *pfbegin, _PIFV *pfend)
{
        int ret = 0;

        while (pfbegin < pfend && ret == 0)
        {
            if (*pfbegin != 0)
                ret = (**pfbegin)();
            ++pfbegin;
        }

        return ret;
}

EXTERN void runUninit()
{
        if (onexitbegin)
        {
                while (--onexitend >= onexitbegin)
                        if (*onexitend != 0)
                                (**onexitend)();
        }

        __init(__xp_a, __xp_z);
        __init(__xt_a, __xt_z);
}

EXTERN int onexitinit()
{
        onexitend = onexitbegin = onexitarray;
        *onexitbegin = 0;
        return 0;
}

#pragma data_seg(".CRT$XIB")      // run onexitinit automatically
__declspec(allocate(".CRT$XIB")) static _PIFV pinit = onexitinit;
#pragma data_seg()
</syntaxhighlight>

== Local Static Variables (GCC Only) ==
When you declare a local static variable, GCC puts a guard around the variable's constructor call. This ensures that only one thread can call the constructor at the same time to initialize it.

''Note that these are only stubs to get the code compiled, and you should implement them yourself. Simply add a mutex-like guard with a test-and-set primitive.''

<syntaxhighlight lang="cpp">
namespace __cxxabiv1 
{
	/* guard variables */

	/* The ABI requires a 64-bit type.  */
	__extension__ typedef int __guard __attribute__((mode(__DI__)));

	extern "C" int __cxa_guard_acquire (__guard *);
	extern "C" void __cxa_guard_release (__guard *);
	extern "C" void __cxa_guard_abort (__guard *);

	extern "C" int __cxa_guard_acquire (__guard *g) 
	{
		return !*(char *)(g);
	}

	extern "C" void __cxa_guard_release (__guard *g)
	{
		*(char *)g = 1;
	}

	extern "C" void __cxa_guard_abort (__guard *)
	{

	}
}
</syntaxhighlight>

The actual code emitted by GCC to call a local static variable's constructor looks something like this:

<syntaxhighlight lang="cpp">
static <type> guard;

if (!guard.first_byte)
{
	if (__cxa_guard_acquire (&guard))
	{
		bool flag = false;

		try
		{
			// Do initialization.
			__cxa_guard_release (&guard);

			flag = true;
			// Register variable for destruction at end of program.
		}

		catch
		{
			if (!flag)
			{
				__cxa_guard_abort (&guard);
			}
		}
	}
}
</syntaxhighlight>

== The Operators 'new' and 'delete' ==
Before you can properly use '''new''' and '''delete''', you have to implement some sort of memory management. You also have to implement both operators (including their array counterparts). '''new''' and '''delete''' respectively allocate and delete memory (much like '''malloc''' and '''free''' in C). Take a look at the [[Memory Management]] article if you would like to know more about this subject.

Every time you call one of the operators '''new''', '''new[]''', '''delete''', or '''delete[]''', the compiler inserts a call to them. The most simple implementation would be to map them to your kernel's '''malloc''' and '''free'''. For example:

<syntaxhighlight lang="cpp">
#include <stddef.h>

void *operator new(size_t size)
{
    return malloc(size);
}

void *operator new[](size_t size)
{
    return malloc(size);
}

void operator delete(void *p)
{
    free(p);
}

void operator delete[](void *p)
{
    free(p);
}
</syntaxhighlight>

You could also let '''new''' use '''calloc''' (allocate and zero). This way, newly allocated memory will always be zeroed (thus, not contain garbage). The standard '''new''' implementations do however not clear the returned memory.

An easy malloc implementation you can port to your OS is [https://github.com/blanham/liballoc/ liballoc]. It only requires basic [[Paging]] (that is, store a list of used and free pages, and have a function to find the next free page) to work.

=== Placement New ===
In C++ (especially in OS code where structures can be found at fixed addresses) it can be useful to construct an object in memory obtained elsewhere. This is accomplished through a technique known as 'placement new'. For example, say you wanted to create an APIC object at address '''0x09FFF0000''', then this snippet of code will use placement new to do the trick:

<syntaxhighlight lang="cpp">
void *apic_address = reinterpret_cast<void *>(0x09FFF0000);
APIC *apic = new (apic_address) APIC;
</syntaxhighlight>

In order to use placement new, you need special overloads of the new and delete operators defined in scope. Fortunately, the required definitions are simple and can be inlined in a header file (the C++ standard puts them in a header called '''new''').

<syntaxhighlight lang="cpp">
inline void *operator new(size_t, void *p)     throw() { return p; }
inline void *operator new[](size_t, void *p)   throw() { return p; }
inline void  operator delete  (void *, void *) throw() { };
inline void  operator delete[](void *, void *) throw() { };
</syntaxhighlight>

The above implementation can potentially be unsafe for allocating memory since your kernel does not mark the memory that was allocated as being used. Placement new is hardly ever used, and if you wish to read an object from a specified address in memory, it is usually easier to create a pointer to that address.

You never call placement delete explicitly (it's only required for certain implementation detail reasons). Instead, you simply invoke your object's destructor explicitly.

<syntaxhighlight lang="cpp">
apic->~APIC();
</syntaxhighlight>

== RTTI (Run-Time Type Information) ==
RTTI is used for '''typeid''' and '''dynamic_cast'''. It requires runtime support as well. Disable it with '''-fno-rtti'''. A kernel has no access to run-time features, which are most likely operating system-dependent. Note that virtual functions work without RTTI.

== Exceptions ==
Another feature that requires run-time support. Disable it with '''-fno-exceptions'''. Exceptions require code to unwind the stack while looking for an appropriate exception handler to handle the exception. Usually, this code is linked in with your C++ application, but in a freestanding kernel the code must be provided manually.

See [[C++ Exception Support]].

== Standard Library ==
Note that the C++ Standard Library (stdlib) is not the same as the C++ Standard Template Library (STL). The STL was designed in 1994 and largely influenced the C++ Standard Library, but it's not a part of the ISO C++ standard. The C++ Standard Library is part of the C++ ISO specification, however, and is what you're using when you use ''std::vector'', ''std::string'', etc. Be wary of misusing the term STL and, ideally, avoid it completely. Anyone using it almost certainly means the C++ stdlib.

You cannot use stdlib functions or classes without porting a stdlib implementation. A lot of existing code depending on the stdlib is OS-dependent, so you must port an stdlib implementation to your OS if you want to use them.

To gain access to the stdlib in your OS you can do either of the following:
* Write your own implementation of a few of the required class templates (std::string, std::list, std::cout, ...).
* Port a stdlib implementation to your OS (e.g. [[STLport]]).

A lot of the stdlib classes require '''new''' and '''delete''' to be implemented in your OS. File access requires your OS to support reading and wrapping. Console functions require your OS to already have working console I/O.

Porting the C++ stdlib (like porting the [[C_Library|C Standard Library]]) does not automatically make your OS able to read from and write to the disk or get data straight from the keyboard. These are simply wrappers around your OS' functions, and must be implemented by in your kernel.

Note that it is generally not a good idea to port the entire stdlib to your kernel, although it is reasonable to port a few class templates, such as <tt>std::vector</tt> and <tt>std::string</tt> if you wish to. As for your user applications: the more the merrier! :)

Here is a list of a the most commonly used stdlib implementations:

* [http://incubator.apache.org/stdcxx/ STDCXX] (a.k.a Apache C++ Standard Library, formally Rogue Wave C++ Standard Library)
* [https://www.dinkumware.com/ Dinkumware C++ Standard Library]
* [https://github.com/microsoft/STL Microsoft C++ Standard Library]
* [http://gcc.gnu.org/libstdc++/ libstdc++] (a.k.a. GNU Standard C++ Library)
* [http://www.stlport.org/ STLport]
* [http://ustl.sourceforge.net/ uSTL]
* [http://libcxx.llvm.org/ libc++] (LLVM C++ Standard library)

== Full C++ Runtime Support Using libgcc And libsupc++ ==
{{Main|Libsupcxx#Full C++ Runtime Support Using libgcc And libsupc++}}

If you want Exceptions, RTTI, new and delete altogether, you should use [[libgcc]] and libsupc++. libgcc contains the unwinder (for exceptions), while libsupc++ contains the C++ support.

You may run into problems with libsupc++, but there are [[GCC_and_Libc++|alternative libraries]].

== Optimizations ==
There are [[C#Things_you_should_know_about_optimizations|things you should know about optimizations]] that also affect C++ because it is an extension of the [[C]] language. You should know about them even if you don't plan to to use the optimizer of your C++ compiler in the near future.

== Links ==
=== Wiki ===
* [[Bare Bones]]
* [[Volatile_(keyword)|Use of the volatile keyword]]
* [[C++ to ASM linkage in GCC|Linking C++ and Assembly (GCC-specific)]]

[[Category:Languages]]
[[Category:C++]]
[[de:C++]]
````

<a id="w001529"></a>

## W001529 — User:Combuster

Bron-ID: W001529
Namespace: User (2)
Revisie: 19430 | Datum: 2016-07-11T19:53:45Z
Bron: https://wiki.osdev.org/User:Combuster
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3ACombuster&oldid=19430

````text
I'm Marcel Sondaar, aka (Com)Buster (commonly seen without () due to software limitations)

I'm a master student as well as a university staff member (Teacher's assistant - giving work colleges) at Utrecht University. Together with my affinity to computers its probably enough cause to end up running around here trying to help people.

My hobby OS is called [http://www.d-rift.nl/combuster/mos3/ MysticOS], which runs currently as a test bed for various useful and the more pointless ideas of mine. Basically its a waste of time rather than a noble attempt at improving the world. Although most of its sources are overcommented and in the public domain for those wanting to learn from it.

As of this writing, I have 15 years of programming experience. My favorite language is BASIC, in which I can safely claim guru status. (And experience has shown that if you don't want to get your butt kicked, try not to apply BASIC stereotypes on me) Of course, I can speak a dozen of other languages:
* Assembly (x86, Atmel, 68k)
* C
* C++
* Java
* Haskell
* Several other specialized languages not worth mentioning

== The books I found useful in OS development settings ==
* Intel Manuals
* Graphics Programming Black Book
* Fundamentals of Computer Graphics
* Computer Networking: A Top-down Approach

And from the university press: 
* Reader Systems architecture
* Reader Distributed Programming
* Reader Program Correctness

== Pages Here ==
* [[User:Combuster/notepad|Random Knowledge]]
* [[User:Combuster/Mach64 Hardware|Notes and reverse engineered info on the ATI Mach64 series]] (Commonly known as 3D Rage)

== My larger contributions ==
* [[VGA Hardware]]
* [[Bootable CD]]
* [[FreeBasic Bare Bones]]
* [[Expanded Main Page]]
* [[Video Signals And Timing]]
* [[Java Primer]]

== Browse ==
{{Special:Prefixindex/User:Combuster/}}
````

<a id="w001530"></a>

## W001530 — User:Walling/Archive/UserPage1

Bron-ID: W001530
Namespace: User (2)
Revisie: 8156 | Datum: 2009-06-04T14:24:16Z
Bron: https://wiki.osdev.org/User:Walling/Archive/UserPage1
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AWalling%2FArchive%2FUserPage1&oldid=8156

````text
__NOTOC__
<div class="plainlinks">

Hi, I'm '''Bjarke Walling'''. Please [http://forum.osdev.org/memberlist.php?mode=viewprofile&u=2204 view my forum profile].

I'm staying in Norway for the moment (looking for a IT job), and not very active in OS development. If you need to concact me, please do so [mailto:bwp@bwp.dk by e-mail].
</div>

== Various stuff ==

=== User pages ===

* [[User:Walling/Sandbox|User sandbox]]
* [[User:Walling/Archive|User archive]]

A listing of all my user pages: {{Special:Prefixindex/User:Walling}}
Discussion: {{Special:Prefixindex/User_talk:Walling}}

=== Possible projects ===

What I could do:

* [[Special:Deadendpages|Remove dead-end pages by creating links]]
* [[Special:Lonelypages|Bring orphaned pages down to a minimum]]
* [[Special:Mostlinked|Create most linked to articles not existing]]
* [[Special:Uncategorizedcategories|Ensure no categories]] [[Special:Uncategorizedpages|and pages are uncategorized]]
* [[:Category:Stubs|Maybe extend stubs]]

=== Bookmarks ===

Pages on OSDev I bookmarked:

<div class="plainlinks">
* [http://www.osdev.org/phpBB2/viewtopic.php?t=12194 The "Singularity" approach ... (forum topic)] - bookmarked 2006-12-16
* [http://www.osdev.org/phpBB2/viewtopic.php?t=12349 new/delete... (forum topic)] - bookmarked 2006-12-16
* [http://www.osdev.org/phpBB2/viewtopic.php?t=12389 Interrupt called from 16 or 32 bit code (own forum topic)] - bookmarked 2006-12-16
</div>

=== Pages I created ===

This is mostly information for myself:

* [[Stack]] - created 2006-12-20
* [[INT]] - created 2006-12-13
* [[Interrupt Vector Table]] - created 2006-12-08
* [[Interrupt Descriptor Table]] - created 2006-12-08
* [[Global Descriptor Table]] - created 2006-12-07

=== Uploaded images ===

* [[:Image:Data stack.png|Data stack.png]] - uploaded 2006-12-20
* [[:Image:Data stack.svg|Data stack.svg]] - uploaded 2006-12-20
````

<a id="w001531"></a>

## W001531 — GDT Tutorial

Bron-ID: W001531
Namespace: Main (0)
Revisie: 30355 | Datum: 2026-03-18T19:41:25Z
Bron: https://wiki.osdev.org/GDT_Tutorial
Revisie-URL: https://wiki.osdev.org/index.php?title=GDT_Tutorial&oldid=30355

````text
{{Rating|1}}
On the [[IA32_Architecture_Family|IA-32]] and [[X86-64|x86-64]] architectures, and more precisely in '''[[Protected Mode]]''' or '''[[Long Mode]]''', [[Interrupt Service Routines]] and a good deal of [[memory management]] are controlled through tables of descriptors. Each descriptor stores information about a single object (e.g. a service routine, a task, a chunk of code or data) the CPU might need at some time. If you try, for instance, to load a new value into a '''[[Segmentation|Segment Register]]''', the CPU needs to perform safety and access control checks to see whether you're actually entitled to access that specific memory area. Once the checks are performed, useful values (such as the lowest and highest addresses) are cached in invisible CPU registers.

On these architectures, there are three of this type of table: The '''[[Global Descriptor Table]]''', the '''[[Local Descriptor Table]]''' and the '''[[Interrupt Descriptor Table]]''' (which supplants the '''[[Interrupt Vector Table]]'''). Each table is defined using their size and '''[[linear address]]''' to the CPU through the '''LGDT''', '''LLDT''', and '''LIDT''' instructions respectively. In almost all use cases, these tables are only placed into memory once, at boot time, and then edited later when needed.

== Survival Glossary ==

; '''[[Segmentation|Segment]]'''
: A logically contiguous chunk of memory with consistent properties (from the CPU's perspective).
; '''Segment Register'''
: A register of your CPU that refers to a segment for a particular purpose ('''CS''', '''DS''', '''SS''', '''ES''') or for general use ('''FS''', '''GS''')
; '''[[Segment Selector]]'''
: A reference to a descriptor, which you can load into a segment register; the selector is an offset into a descriptor table pointing to one of its entries. These entries are typically 8 bytes long, therefore bits 3 and up only declare the descriptor table entry offset, while bit 2 specifies if this selector is a GDT or LDT selector (LDT - bit set, GDT - bit cleared), and bits 0 - 1 declare the ring level that needs to correspond to the descriptor table entry's DPL field. If it doesn't, a General Protection Fault occurs; if it does correspond then the CPL level of the selector used is changed accordingly.
; '''[[Global_Descriptor_Table#Segment_Descriptor|Segment Descriptor]]'''
: An entry in a descriptor table. These are a binary data structure that tells the CPU the attributes of a given segment.

== What to Put In a GDT ==

=== Basics ===

For the sake of sanity, you should always have these items stored in your GDT:

* Entry 0 in a descriptor table, or the '''Null Descriptor''', is never referenced by the processor, and should always contain no data. Certain emulators, like Bochs, will complain about limit exceptions if you do not have one present. Some use this descriptor to store a pointer to the GDT itself (to use with the LGDT instruction). The null descriptor is 8 bytes wide and the pointer is 6 bytes wide so it might just be the perfect place for this.
* A DPL 0 '''Code Segment''' descriptor (for your kernel)
* A '''Data Segment''' descriptor (writing to code segments is not allowed)
* A '''[[Task State Segment]]''' segment descriptor (its very useful to have at least one)
* Room for more segments if you need them (e.g. user-level, [[LDT]]s, more TSS, whatever)

=== Flat / Long Mode Setup ===

If you do not desire to use '''[[Segmentation]]''' to separate memory into protected areas, you can get away with using only a few segment descriptors. One reason may be that you desire to only use paging to protect memory. As well, this model is ''strictly enforced'' in '''[[Long Mode]]''', as the base and limit values are ignored.

In this scenario, the only '''[[Global_Descriptor_Table#Segment_Descriptor|Segment Descriptors]]''' necessary are the '''Null Descriptor''', and one descriptor for each combination of privilege level, segment type, and execution mode desired, as well as system descriptors. Usually this will consist of the one code and one data segment for kernel and user mode, and a '''[[Task State Segment]]'''.

{|class="wikitable" style="display: inline-table;"
|+ 32-bit
! Offset !! Use !! Content
|-
| 0x0000 || Null Descriptor || <tt>Base = 0<br>Limit = 0x00000000<br>Access Byte = 0x00<br>Flags = 0x0</tt>
|-
| 0x0008 || Kernel Mode Code Segment || <tt>Base = 0<br>Limit = 0xFFFFF<br>Access Byte = 0x9A<br>Flags = 0xC</tt>
|-
| 0x0010 || Kernel Mode Data Segment || <tt>Base = 0<br>Limit = 0xFFFFF<br>Access Byte = 0x92<br>Flags = 0xC</tt>
|-
| 0x0018 || User Mode Code Segment || <tt>Base = 0<br>Limit = 0xFFFFF<br>Access Byte = 0xFA<br>Flags = 0xC</tt>
|-
| 0x0020 || User Mode Data Segment || <tt>Base = 0<br>Limit = 0xFFFFF<br>Access Byte = 0xF2<br>Flags = 0xC</tt>
|-
| 0x0028 || Task State Segment || <tt>Base = &TSS<br>Limit = sizeof(TSS)-1<br>Access Byte = 0x89<br>Flags = 0x0</tt>
|}

{|class="wikitable" style="display: inline-table;"
|+ 64-bit
! Offset !! Use !! Content
|-
| 0x0000 || Null Descriptor || <tt>Base = 0<br>Limit = 0x00000000<br>Access Byte = 0x00<br>Flags = 0x0</tt>
|-
| 0x0008 || Kernel Mode Code Segment || <tt>Base = 0<br>Limit = 0xFFFFF<br>Access Byte = 0x9A<br>Flags = 0xA</tt>
|-
| 0x0010 || Kernel Mode Data Segment || <tt>Base = 0<br>Limit = 0xFFFFF<br>Access Byte = 0x92<br>Flags = 0xC</tt>
|-
| 0x0018 || User Mode Data Segment || <tt>Base = 0<br>Limit = 0xFFFFF<br>Access Byte = 0xF2<br>Flags = 0xC</tt>
|-
| 0x0020 || User Mode Code Segment || <tt>Base = 0<br>Limit = 0xFFFFF<br>Access Byte = 0xFA<br>Flags = 0xA</tt>
|-
| 0x0028 || Task State Segment<br>('''[[Global Descriptor Table#Long Mode System Segment Descriptor|64-bit System Segment]]''') || <tt>Base = &TSS<br>Limit = sizeof(TSS)-1<br>Access Byte = 0x89<br>Flags = 0x0</tt>
|}

=== Small Kernel Setup ===

If you desire to separate memory into protected areas of code and data, you will have to set the '''Base''' and '''Limit''' value of each entry in the table to the desired format.

For example, you may want to have two segments, a 4MiB Code Segment starting at 4MiB, and a 4MiB Data Segment starting at 8MiB, both accessible only to Ring 0. In that case, your GDT may look like this:

{|class="wikitable" style="display: inline-table;"
|+ Small Kernel
! Offset !! Use !! Content
|-
| 0x0000 || Null Descriptor || <tt>Base = 0<br>Limit = 0x00000000<br>Access Byte = 0x00<br>Flags = 0x0</tt>
|-
| 0x0008 || Kernel Mode Code Segment || <tt>Base = 0x00400000<br>Limit = 0x003FFFFF<br>Access Byte = 0x9A<br>Flags = 0xC</tt>
|-
| 0x0010 || Kernel Mode Data Segment || <tt>Base = 0x00800000<br>Limit = 0x003FFFFF<br>Access Byte = 0x92<br>Flags = 0xC</tt>
|-
| 0x0018 || Task State Segment || <tt>Base = &TSS<br>Limit = sizeof(TSS)-1<br>Access Byte = 0x89<br>Flags = 0x0</tt>
|}

That means whatever you load at physical address 4 MiB will appear as code at '''CS:0''' and what you load at physical address 8 MiB will appear as data at '''DS:0'''.

This specifically is not a recommended design, but shows how to think about using the '''GDT''' to define separated segments.

=== SYSENTER / SYSEXIT ===

If you are using the Intel '''SYSENTER''' / '''SYSEXIT''' routines, the '''GDT''' must contain four special entries, the first one pointed to by the value in the '''IA32_SYSENTER_CS''' '''[[Model Specific Registers|Model Specific Register]]''' (MSR 0x0174).

For more information, see the sections on '''SYSENTER''' and '''SYSEXIT''' in '''Chapter 4.3: Instructions (M-U)''' of the Intel Software Developer Manual, Volume 2-B.

{|class="wikitable"
|+ GDT
! Offset !! Use
|-
| Preceding Entries || Null Descriptor<br>Kernel Segments<br>etc.
|-
| IA32_SYSENTER_CS + 0x0000 || DPL 0 Code Segment<br>'''SYSENTER''' Code
|-
| IA32_SYSENTER_CS + 0x0008 || DPL 0 Data Segment<br>'''SYSENTER''' Stack
|-
| IA32_SYSENTER_CS + 0x0010 || DPL 3 Code Segment<br>'''SYSEXIT''' Code
|-
| IA32_SYSENTER_CS + 0x0018 || DPL 3 Data Segment<br>'''SYSEXIT''' Stack
|-
| Successive Entries || Any Other Descriptors
|}

The actual values stored in these segments will depend on your system design.

== How to Set Up The GDT ==

=== Disable Interrupts ===

If they're enabled, ''be absolutely sure'' to turn them off or you could run into undesired behavior and exceptions. This can be achieved through the '''CLI''' assembly instruction.

=== Filling the Table ===

The above structure of the '''GDT''' doesn't show you how to write entries in the correct format. The actual structure of descriptors is a little messy for backwards compatibility with the 286's '''GDT'''. Base address are split across three different fields and you cannot encode any limit you want.

<syntaxhighlight lang="c">
void encodeGdtEntry(uint8_t *target, struct GDT source)
{
    // Check the limit to make sure that it can be encoded
    if (source.limit > 0xFFFFF) {kerror("GDT cannot encode limits larger than 0xFFFFF");}
    
    // Encode the limit
    target[0] = source.limit & 0xFF;
    target[1] = (source.limit >> 8) & 0xFF;
    target[6] = (source.limit >> 16) & 0x0F;
    
    // Encode the base
    target[2] = source.base & 0xFF;
    target[3] = (source.base >> 8) & 0xFF;
    target[4] = (source.base >> 16) & 0xFF;
    target[7] = (source.base >> 24) & 0xFF;
    
    // Encode the access byte
    target[5] = source.access_byte;
    
    // Encode the flags
    target[6] |= (source.flags << 4);
}
</syntaxhighlight>

In order to fill your table, you will want to use this function once for each entry, with <tt>*target</tt> pointing to the logical address of the '''Segment Descriptor''' and <tt>source</tt> being a struct of your design containing the necessary information.

You can hard-code values in the '''GDT''' rather than converting them at runtime, of course.

=== Telling the CPU Where the Table Is ===

Some assembly is required here. While you could use [[inline assembly]], the memory packing expected by the '''LGDT''' and '''LIDT''' instructions makes it much easier to write a small assembly routine instead. As said above, you'll use '''LGDT''' instruction to load the base address and the limit of the GDT. Since the base address should be a linear address, you'll need a bit of tweaking depending of your current [[MMU]] setup.

==== Real Mode ====

The linear address should here be computed as <tt>segment * 16 + offset</tt>. <tt>GDT</tt> and <tt>GDT_end</tt> are assumed to be symbols in the current data segment.

<syntaxhighlight lang="asm">
gdtr DW 0 ; For limit storage
     DD 0 ; For base storage

setGdt:
   XOR   EAX, EAX
   MOV   AX, DS
   SHL   EAX, 4
   ADD   EAX, ''GDT''
   MOV   [gdtr + 2], eax
   MOV   EAX, ''GDT_end''
   SUB   EAX, ''GDT''
   MOV   [gdtr], AX
   LGDT  [gdtr]
   RET
</syntaxhighlight>

==== Protected Mode, Flat Model ====

"Flat" meaning the base of your Data Segment is 0 (regardless of whether '''[[Paging]]''' is enabled). This is the case if your code has just been booted by [[GRUB]], for instance. In the '''[[System V ABI]]''', arguments are passed on reverse order in the stack, so a function that can be called as <tt>setGdt(limit, base)</tt> might look like the following example code.

<syntaxhighlight lang="asm">
gdtr DW 0 ; For limit storage
     DD 0 ; For base storage

setGdt:
   MOV   AX, [esp + 4]
   MOV   [gdtr], AX
   MOV   EAX, [ESP + 8]
   MOV   [gdtr + 2], EAX
   LGDT  [gdtr]
   RET
</syntaxhighlight>

==== Protected Mode, Non-Flat Model ====

If your data segment has a non-zero base, you'll have to adjust the instructions of the sequence above to include the ability to add the base offset of your data segment, which should be a known value to you. You can pass it in as an argument and call this function as <tt>setGdt(limit, base, offset)</tt>.

<syntaxhighlight lang="asm">
gdtr DW 0 ; For limit storage
     DD 0 ; For base storage

setGdt:
   MOV   AX, [esp + 4]
   MOV   [gdtr], AX
   MOV   EAX, [ESP + 8]
   ADD   EAX, [ESP + 12]
   MOV   [gdtr + 2], EAX
   LGDT  [gdtr]
   RET
</syntaxhighlight>

==== Long Mode ====

In '''[[Long Mode]]''', the length of the '''Base''' field is 8 bytes, rather than 4. As well, the '''[[System V ABI]]''' passes the first two arguments via the '''RDI''' and '''RSI''' registers. Thus, this example code can be called as <tt>setGdt(limit, base)</tt>. As well, only a flat model is possible in long mode, so no considerations have to be made otherwise.

<syntaxhighlight lang="asm">
gdtr DW 0 ; For limit storage
     DQ 0 ; For base storage

setGdt:
   MOV   [gdtr], DI
   MOV   [gdtr+2], RSI
   LGDT  [gdtr]
   RET
</syntaxhighlight>

=== Reload Segment Registers ===

Whatever you do with the '''GDT''' has no effect on the CPU until you load new '''Segment Selectors''' into '''Segment Registers'''. For most of these registers, the process is as simple as using '''MOV''' instructions, but changing the '''CS''' register requires code resembling a jump or call to elsewhere, as this is the only way its value is meant to be changed.

==== Protected Mode ====

In this case, reloading '''CS''' is as simple as performing a far jump to the required segment, directly after the jump instruction:

<syntaxhighlight lang="asm">
reloadSegments:
   ; Reload CS register containing code selector:
   JMP   0x08:.reload_CS ; 0x08 is a stand-in for your code segment
.reload_CS:
   ; Reload data segment registers:
   MOV   AX, 0x10 ; 0x10 is a stand-in for your data segment
   MOV   DS, AX
   MOV   ES, AX
   MOV   FS, AX
   MOV   GS, AX
   MOV   SS, AX
   RET
</syntaxhighlight>

An explanation of the above code can be found [http://stackoverflow.com/questions/23978486/far-jump-in-gdt-in-bootloader here].

==== Long Mode ====

In '''[[Long Mode]]''' the process of changing '''CS''' is not simple as far jumps cannot be used. Using a far return is recommended instead:

<syntaxhighlight lang = "asm">
reloadSegments:
   ; Reload CS register:
   PUSH 0x08                 ; Push code segment to stack, 0x08 is a stand-in for your code segment
   LEA RAX, [rel .reload_CS] ; Load address of .reload_CS into RAX
   PUSH RAX                  ; Push this value to the stack
   RETFQ                     ; Perform a far return, RETFQ or LRETQ depending on syntax
.reload_CS:
   ; Reload data segment registers
   MOV   AX, 0x10 ; 0x10 is a stand-in for your data segment
   MOV   DS, AX
   MOV   ES, AX
   MOV   FS, AX
   MOV   GS, AX
   MOV   SS, AX
   RET
</syntaxhighlight>

== The LDT ==

Much like the GDT (global descriptor table), the LDT (''local'' descriptor table) contains descriptors for memory segments description, call gates, etc. The good thing with the LDT is that each task can have its own LDT and that the processor will automatically switch to the right LDT when you use hardware task switching.

Since its content may be different in each task, the LDT is not a suitable place to put system stuff such as TSS or other LDT descriptors: Those are the sole property of the GDT. Since it is meant to change often, the command used for loading an LDT is a bit different from the GDT and IDT loading. Rather than giving directly the LDT's base address and size, those parameters are stored in a descriptor of the GDT (with proper "LDT" type) and the selector of that entry is given.

                GDTR (base + limit)
               +-- GDT ------------+
               |                   |
 SELECTOR ---> [LDT descriptor     ]----> LDTR (base + limit)
               |                   |     +-- LDT ------------+
               |                   |     |                   |
              ...                 ...   ...                 ...
               +-------------------+     +-------------------+

Note that with 386+ processors, the paging has made LDT almost obsolete, and there's no longer need for multiple LDT descriptors, so you can almost safely ignore the LDT for OS developing, unless you have by design many different segments to store.

== The IDT and why it's needed ==

As said above, the IDT (Interrupt Descriptor Table) loads much the same way as the GDT and its structure is roughly the same except that it only contains gates and not segments. Each gate gives a full reference to a piece of code (code segment, privilege level and offset to the code in that segment) that is now bound to a number between 0 and 255 (the slot in the IDT).

The IDT will be one of the first things to be enabled in your kernel sequence, so that you can catch hardware exceptions, listen to external events, etc. See [[Interrupts]] for more information about the interrupts of X86 family.

== Some stuff to make your life easy ==

Tool for easily creating GDT entries.
<syntaxhighlight lang="c">
// Used for creating GDT segment descriptors in 64-bit integer form.
 
#include <stdio.h>
#include <stdint.h>
 
// Each define here is for a specific flag in the descriptor.
// Refer to the intel documentation for a description of what each one does.
#define SEG_DESCTYPE(x)  ((x) << 0x04) // Descriptor type (0 for system, 1 for code/data)
#define SEG_PRES(x)      ((x) << 0x07) // Present
#define SEG_SAVL(x)      ((x) << 0x0C) // Available for system use
#define SEG_LONG(x)      ((x) << 0x0D) // Long mode
#define SEG_SIZE(x)      ((x) << 0x0E) // Size (0 for 16-bit, 1 for 32)
#define SEG_GRAN(x)      ((x) << 0x0F) // Granularity (0 for 1B - 1MB, 1 for 4KB - 4GB)
#define SEG_PRIV(x)     (((x) &  0x03) << 0x05)   // Set privilege level (0 - 3)
 
#define SEG_DATA_RD        0x00 // Read-Only
#define SEG_DATA_RDA       0x01 // Read-Only, accessed
#define SEG_DATA_RDWR      0x02 // Read/Write
#define SEG_DATA_RDWRA     0x03 // Read/Write, accessed
#define SEG_DATA_RDEXPD    0x04 // Read-Only, expand-down
#define SEG_DATA_RDEXPDA   0x05 // Read-Only, expand-down, accessed
#define SEG_DATA_RDWREXPD  0x06 // Read/Write, expand-down
#define SEG_DATA_RDWREXPDA 0x07 // Read/Write, expand-down, accessed
#define SEG_CODE_EX        0x08 // Execute-Only
#define SEG_CODE_EXA       0x09 // Execute-Only, accessed
#define SEG_CODE_EXRD      0x0A // Execute/Read
#define SEG_CODE_EXRDA     0x0B // Execute/Read, accessed
#define SEG_CODE_EXC       0x0C // Execute-Only, conforming
#define SEG_CODE_EXCA      0x0D // Execute-Only, conforming, accessed
#define SEG_CODE_EXRDC     0x0E // Execute/Read, conforming
#define SEG_CODE_EXRDCA    0x0F // Execute/Read, conforming, accessed
 
#define GDT_CODE_PL0 SEG_DESCTYPE(1) | SEG_PRES(1) | SEG_SAVL(0) | \
                     SEG_LONG(0)     | SEG_SIZE(1) | SEG_GRAN(1) | \
                     SEG_PRIV(0)     | SEG_CODE_EXRD
 
#define GDT_DATA_PL0 SEG_DESCTYPE(1) | SEG_PRES(1) | SEG_SAVL(0) | \
                     SEG_LONG(0)     | SEG_SIZE(1) | SEG_GRAN(1) | \
                     SEG_PRIV(0)     | SEG_DATA_RDWR
 
#define GDT_CODE_PL3 SEG_DESCTYPE(1) | SEG_PRES(1) | SEG_SAVL(0) | \
                     SEG_LONG(0)     | SEG_SIZE(1) | SEG_GRAN(1) | \
                     SEG_PRIV(3)     | SEG_CODE_EXRD
 
#define GDT_DATA_PL3 SEG_DESCTYPE(1) | SEG_PRES(1) | SEG_SAVL(0) | \
                     SEG_LONG(0)     | SEG_SIZE(1) | SEG_GRAN(1) | \
                     SEG_PRIV(3)     | SEG_DATA_RDWR
 
void
create_descriptor(uint32_t base, uint32_t limit, uint16_t flag)
{
    uint64_t descriptor;
 
    // Create the high 32 bit segment
    descriptor  =  limit       & 0x000F0000;         // set limit bits 19:16
    descriptor |= (flag <<  8) & 0x00F0FF00;         // set type, p, dpl, s, g, d/b, l and avl fields
    descriptor |= (base >> 16) & 0x000000FF;         // set base bits 23:16
    descriptor |=  base        & 0xFF000000;         // set base bits 31:24
 
    // Shift by 32 to allow for low part of segment
    descriptor <<= 32;
 
    // Create the low 32 bit segment
    descriptor |= base  << 16;                       // set base bits 15:0
    descriptor |= limit  & 0x0000FFFF;               // set limit bits 15:0
 
    printf("0x%.16llX\n", descriptor);
}
 
int
main(void)
{
    create_descriptor(0, 0, 0);
    create_descriptor(0, 0x000FFFFF, (GDT_CODE_PL0));
    create_descriptor(0, 0x000FFFFF, (GDT_DATA_PL0));
    create_descriptor(0, 0x000FFFFF, (GDT_CODE_PL3));
    create_descriptor(0, 0x000FFFFF, (GDT_DATA_PL3));
 
    return 0;
}
 
</syntaxhighlight>

== See Also ==

=== Articles ===
* [[Global Descriptor Table]]
* http://web.archive.org/web/20190424213806/http://www.osdever.net/tutorials/view/the-world-of-protected-mode - how to set up GDT in assembler

=== Threads ===

=== External Links ===

[[Category:Tutorials]]
[[Category:X86 CPU]]
[[Category:Memory Segmentation]]
````

<a id="w001533"></a>

## W001533 — Global Descriptor Table

Bron-ID: W001533
Namespace: Main (0)
Revisie: 29554 | Datum: 2025-06-16T22:22:03Z
Bron: https://wiki.osdev.org/Global_Descriptor_Table
Revisie-URL: https://wiki.osdev.org/index.php?title=Global_Descriptor_Table&oldid=29554

````text
The '''Global Descriptor Table''' ('''GDT''') is a binary data structure specific to the [[IA32_Architecture_Family|IA-32]] and [[X86-64|x86-64]] architectures. It contains entries telling the CPU about memory [[Segmentation|segments]]. A similar [[Interrupt Descriptor Table]] exists containing [[task]] and [[Interrupts|interrupt]] descriptors.

It is recommended to read the [[GDT Tutorial]].

== GDTR ==

The '''GDT''' is pointed to by the value in the '''GDTR''' register. This is loaded using the '''LGDT''' assembly instruction, whose argument is a pointer to a '''GDT Descriptor''' structure:

{| class="wikitable"
|+ GDT Descriptor (GDTR)
!style="width: 66%; text-align: left;" |79 (64-bit mode) <br> 47 (32-bit mode)&nbsp;&nbsp;&nbsp;<span style="float: right;">16</span>
!style="width: 34%; text-align: left; vertical-align: bottom;" |15&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|-
|'''Offset'''<br>63 (64-bit mode) <br> 31 (32-bit mode)&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|'''Size'''<br><br>15&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|}
* '''Size:''' The size of the table in bytes subtracted by 1. This subtraction occurs because the maximum value of '''Size''' is 65535, while the '''GDT''' can be up to 65536 bytes in length (8192 entries). Further, no '''GDT''' can have a size of 0 bytes.
* '''Offset:''' The linear address of the '''GDT''' (not the physical address, paging applies).

Note that the amount of data loaded by '''LGDT''' differs in 32-bit and 64-bit modes, the offset is 4 bytes long in 32-bit mode and 8 bytes long in 64-bit mode.

For more information, see '''Section 2.4.1: Global Descriptor Table Register (GDTR)''' and '''Figure 2-6: Memory Management Registers''' of the Intel Software Developer Manual, Volume 3-A.

== Table ==

The entries in the '''GDT''' are 8 bytes long and form a table like this:

{|class="wikitable"
|+ Global Descriptor Table
! Address !! Content
|-
| GDTR Offset + 0 || Null
|-
| GDTR Offset + 8 || Entry 1
|-
| GDTR Offset + 16 || Entry 2
|-
| GDTR Offset + 24 || Entry 3
|- style="text-align: center;"
| '''...''' || '''...'''
|}

The first entry in the '''GDT''' (Entry 0) should always be null and subsequent entries should be used instead.

Entries in the table are accessed by '''[[Segment Selector|Segment Selectors]]''', which are loaded into '''[[Segmentation]]''' registers either by assembly instructions or by hardware functions such as '''[[Interrupts]]'''.

== Segment Descriptor ==

Each entry in the table has a complex structure:
{| class="wikitable"
|+ Segment Descriptor
!style="width: 20%; text-align: left;"|63&nbsp;&nbsp;&nbsp;<span style="float: right;">56</span>
!style="width: 12.5%; text-align: left;"|55&nbsp;&nbsp;&nbsp;<span style="float: right;">52</span>
!style="width: 12.5%; text-align: left;"|51&nbsp;&nbsp;&nbsp;<span style="float: right;">48</span>
!style="width: 25%; text-align: left;"|47&nbsp;&nbsp;&nbsp;<span style="float: right;">40</span>
!style="width: 25%; text-align: left;"|39&nbsp;&nbsp;&nbsp;<span style="float: right;">32</span>
|-
|'''Base'''<br>31&nbsp;&nbsp;&nbsp;<span style="float: right;">24</span>
|'''Flags'''<br>3&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|'''Limit'''<br>19&nbsp;&nbsp;&nbsp;<span style="float: right;">16</span>
|'''Access Byte'''<br>7&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|'''Base'''<br>23&nbsp;&nbsp;&nbsp;<span style="float: right;">16</span>
|-
!colspan=3 style="text-align: left;"|31 &nbsp;&nbsp;<span style="float: right;">16</span>
!colspan=2 style="text-align: left;"|15 &nbsp;&nbsp;<span style="float: right;">0</span>
|-
|colspan=3|'''Base'''<br>15&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|colspan=2|'''Limit'''<br>15&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|}

* '''Base:''' A 32-bit value containing the linear address where the segment begins.
* '''Limit:''' A 20-bit value, tells the maximum addressable unit, either in 1 byte units, or in 4KiB pages. Hence, if you choose page granularity and set the '''Limit''' value to 0xFFFFF the segment will span the full 4 GiB address space in 32-bit mode.

In 64-bit mode, the '''Base''' and '''Limit''' values are ignored, each descriptor covers the entire linear address space regardless of what they are set to.

For more information, see '''Section 3.4.5: Segment Descriptors''' and '''Figure 3-8: Segment Descriptor''' of the Intel Software Developer Manual, Volume 3-A.

{| class="wikitable"
|+ Access Byte
|-
!style="width: 12.5%;"|7
!style="width: 12.5%;"|6
!style="width: 12.5%;"|5
!style="width: 12.5%;"|4
!style="width: 12.5%;"|3
!style="width: 12.5%;"|2
!style="width: 12.5%;"|1
!style="width: 12.5%;"|0
|-
|'''P''' ||colspan=2|'''DPL''' || '''S''' || '''E''' || '''DC''' || '''RW''' || '''A'''
|}

* '''P:''' Present bit. Allows an entry to refer to a valid segment. Must be set ('''1''') for any valid segment.
* '''DPL:''' Descriptor privilege level field. Contains the [[Security#Rings|CPU Privilege level]] of the segment. '''0''' = highest privilege (kernel), '''3''' = lowest privilege (user applications).
* '''S:''' Descriptor type bit. If clear ('''0''') the descriptor defines a system segment (eg. a [[Task_State_Segment|Task State Segment]]). If set ('''1''') it defines a code or data segment.
* '''E:''' Executable bit. If clear ('''0''') the descriptor defines a data segment. If set ('''1''') it defines a code segment which can be executed from.
* '''DC:''' Direction bit/Conforming bit.
** For data selectors: Direction bit. If clear ('''0''') the segment grows up. If set ('''1''') the segment [[Expand_Down|grows down]], ie. the '''Offset''' has to be greater than the '''Limit'''.
** For code selectors: Conforming bit.
*** If clear ('''0''') code in this segment can only be executed from the ring set in '''DPL'''.
*** If set ('''1''') code in this segment can be executed from an equal or lower privilege level. For example, code in ring 3 can far-jump to ''conforming'' code in a ring 2 segment. The '''DPL''' field represent the highest privilege level that is allowed to execute the segment. For example, code in ring 0 cannot far-jump to a conforming code segment where '''DPL''' is 2, while code in ring 2 and 3 can.  Note that the privilege level remains the same, ie. a far-jump from ring 3 to a segment with a '''DPL''' of 2 remains in ring 3 after the jump.
* '''RW:''' Readable bit/Writable bit.
** For code segments: Readable bit. If clear ('''0'''), read access for this segment is not allowed. If set ('''1''') read access is allowed. Write access is never allowed for code segments.
** For data segments: Writeable bit. If clear ('''0'''), write access for this segment is not allowed. If set ('''1''') write access is allowed. Read access is always allowed for data segments.
* '''A:''' Accessed bit. The CPU will set it when the segment is accessed unless set to '''1''' in advance. This means that in case the GDT descriptor is stored in read only pages and this bit is set to '''0''', the CPU trying to set this bit will trigger a page fault. Best left set to '''1''' unless otherwise needed.

{| class="wikitable"
|+ Flags
|-
!style="width: 25%;"|3
!style="width: 25%;"|2
!style="width: 25%;"|1
!style="width: 25%;"|0
|-
|'''G''' || '''DB''' || '''L'''
|Reserved
|}

* '''G:''' Granularity flag, indicates the size the '''Limit''' value is scaled by. If clear ('''0'''), the '''Limit''' is in 1 Byte blocks (byte granularity). If set ('''1'''), the '''Limit''' is in 4 KiB blocks (page granularity).
* '''DB:''' Size flag. If clear ('''0'''), the descriptor defines a 16-bit protected mode segment. If set ('''1''') it defines a 32-bit protected mode segment. A GDT can have both 16-bit and 32-bit selectors at once.
* '''L:''' Long-mode code flag. If set ('''1'''), the descriptor defines a 64-bit code segment. When set, '''DB''' should always be clear. For any other type of segment (other code types or any data segment), it should be clear ('''0''').

== System Segment Descriptor ==

For system segments, such as those defining a '''[[Task State Segment]]''' or '''[[Local Descriptor Table]]''', the format of the '''Access Byte''' differs slightly, in order to define different types of system segments rather than code and data segments.

For more information, see '''Section 3.5: System Descriptor Types''' and '''Figure 3-2: System-Segment and Gate-Descriptor Types''' of the Intel Software Developer Manual, Volume 3-A.

{| class="wikitable"
|+ Access Byte
|-
!style="width: 12.5%;"|7
!style="width: 12.5%;"|6
!style="width: 12.5%;"|5
!style="width: 12.5%;"|4
!style="width: 12.5%;"|3
!style="width: 12.5%;"|2
!style="width: 12.5%;"|1
!style="width: 12.5%;"|0
|-
|'''P'''&nbsp;&nbsp;&nbsp; ||colspan=2|'''DPL''' || '''S''' || colspan=4|'''Type'''
|}

* '''Type:''' Type of system segment.

Types available in 32-bit protected mode:
* '''0x1:''' 16-bit TSS (Available)
* '''0x2:''' LDT
* '''0x3:''' 16-bit TSS (Busy)
* '''0x9:''' 32-bit TSS (Available)
* '''0xB:''' 32-bit TSS (Busy)

Types available in Long Mode:
* '''0x2:''' LDT
* '''0x9:''' 64-bit TSS (Available)
* '''0xB:''' 64-bit TSS (Busy)

== Long Mode System Segment Descriptor ==

For a '''[[Task State Segment]]''' or '''[[Local Descriptor Table]]''' in '''[[Long Mode]]''', the format of a '''Segment Descriptor''' differs to ensure that the '''Base''' value can contain a 64-bit '''[[Linear Address]]'''. It takes up the space in the table of two usual entries, in a little endian format, such that the lower half of this entry precedes the higher half in the table.

For more information, see '''Section 9.2.3: TSS Descriptor in 64-bit Mode''' and '''Figure 9-4: Format of TSS and LDT Descriptors in 64-bit Mode''' of the Intel Software Developer Manual, Volume 3-A.

{| class="wikitable" style="display: inline-table;"
|+ 64-bit System Segment Descriptor
|-
!colspan=5 style="text-align: left;"|127 &nbsp;&nbsp;<span style="float: right;">96</span>
|-
|colspan=5|Reserved
|-
!colspan=5 style="text-align: left;"|95 &nbsp;&nbsp;<span style="float: right;">64</span>
|-
|colspan=5|'''Base'''<br>63&nbsp;&nbsp;&nbsp;<span style="float: right;">32</span>
|-
!style="width: 20%; text-align: left;"|63&nbsp;&nbsp;&nbsp;<span style="float: right;">56</span>
!style="width: 12.5%; text-align: left;"|55&nbsp;&nbsp;&nbsp;<span style="float: right;">52</span>
!style="width: 12.5%; text-align: left;"|51&nbsp;&nbsp;&nbsp;<span style="float: right;">48</span>
!style="width: 25%; text-align: left;"|47&nbsp;&nbsp;&nbsp;<span style="float: right;">40</span>
!style="width: 25%; text-align: left;"|39&nbsp;&nbsp;&nbsp;<span style="float: right;">32</span>
|-
|'''Base'''<br>31&nbsp;&nbsp;&nbsp;<span style="float: right;">24</span>
|'''Flags'''<br>3&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|'''Limit'''<br>19&nbsp;&nbsp;&nbsp;<span style="float: right;">16</span>
|'''Access Byte'''<br>7&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|'''Base'''<br>23&nbsp;&nbsp;&nbsp;<span style="float: right;">16</span>
|-
!colspan=3 style="text-align: left;"|31 &nbsp;&nbsp;<span style="float: right;">16</span>
!colspan=2 style="text-align: left;"|15 &nbsp;&nbsp;<span style="float: right;">0</span>
|-
|colspan=3|'''Base'''<br>15&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|colspan=2|'''Limit'''<br>15&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|}

== See Also ==
=== Articles ===
* [[GDT Tutorial]]
* [[Getting to Ring 3]]
* [[Segmentation]]
* http://www.osdever.net/tutorials/view/the-world-of-protected-mode - how to set up GDT in assembler

=== External references ===

* [http://files.osdev.org/mirrors/geezer/os/pm.htm Protected Mode tutorial]
* [http://www.intel.com/design/processor/manuals/253668.pdf Intel® 64 and IA-32 Architectures Software Developer’s Manual, Volume 3A:. System Programming Guide, Part 1 (order number 253668)] chapter 2.4

[[Category:X86 CPU]]
[[Category:Memory Segmentation]]
[[de:Global Descriptor Table]]
````

<a id="w001534"></a>

## W001534 — GDT

Bron-ID: W001534
Namespace: Main (0)
Revisie: 1933 | Datum: 2006-12-07T20:46:39Z
Bron: https://wiki.osdev.org/GDT
Revisie-URL: https://wiki.osdev.org/index.php?title=GDT&oldid=1933
Doorverwijzing: Global Descriptor Table

````text
#REDIRECT [[Global Descriptor Table]]
````

<a id="w001535"></a>

## W001535 — Category talk:System Initialization (x86)

Bron-ID: W001535
Namespace: Category talk (15)
Revisie: 1936 | Datum: 2006-12-07T20:53:10Z
Bron: https://wiki.osdev.org/Category_talk:System_Initialization_(x86)
Revisie-URL: https://wiki.osdev.org/index.php?title=Category_talk%3ASystem_Initialization_%28x86%29&oldid=1936

````text
== Article? ==

Shouldn't this be an article, not a category. Or is it just me missing out a point? :-) --[[User:Walling|Walling]] 14:53, 7 December 2006 (CST)
````

<a id="w001536"></a>

## W001536 — Memory Management

Bron-ID: W001536
Namespace: Main (0)
Revisie: 1938 | Datum: 2006-12-07T20:56:29Z
Bron: https://wiki.osdev.org/Memory_Management
Revisie-URL: https://wiki.osdev.org/index.php?title=Memory_Management&oldid=1938
Doorverwijzing: Memory management

````text
#REDIRECT [[Memory management]]
````

<a id="w001537"></a>

## W001537 — Global descriptor table

Bron-ID: W001537
Namespace: Main (0)
Revisie: 1940 | Datum: 2006-12-07T21:17:00Z
Bron: https://wiki.osdev.org/Global_descriptor_table
Revisie-URL: https://wiki.osdev.org/index.php?title=Global_descriptor_table&oldid=1940
Doorverwijzing: Global Descriptor Table

````text
#REDIRECT [[Global Descriptor Table]]
````

<a id="w001538"></a>

## W001538 — Segment

Bron-ID: W001538
Namespace: Main (0)
Revisie: 1941 | Datum: 2006-12-07T21:41:13Z
Bron: https://wiki.osdev.org/Segment
Revisie-URL: https://wiki.osdev.org/index.php?title=Segment&oldid=1941
Doorverwijzing: Segmentation

````text
#REDIRECT [[Segmentation]]
````

<a id="w001540"></a>

## W001540 — Troubleshooting

Bron-ID: W001540
Namespace: Main (0)
Revisie: 28507 | Datum: 2023-12-13T06:59:56Z
Bron: https://wiki.osdev.org/Troubleshooting
Revisie-URL: https://wiki.osdev.org/index.php?title=Troubleshooting&oldid=28507

````text
{{Tone}}
== Providing a basic debugging environment ==
=== Exception handlers ===
The first thing ever to do is to implement a reliable 'exception handler' which will tell you what went wrong. Under an emulator like [[Bochs]], the absence of such a handler will lead to a '3rd Exception without resolution' panic message (a.k.a [[Triple Fault]]), if the emulator is configured to do so... On bare hardware, it will simply reset your computer with a laconic 'bip'.

Every time the CPU is unable to call some exception handler, it tries to execute the [[Double Fault]] exception handler. If it fails to call it either, a [[Triple Fault]] occurs. Also keep in mind that exceptions cannot be masked, so either your code is perfect or you need exception handlers. Also keep in mind that when you run applications '''''their''''' code must be perfect without exception handlers, so it's a good idea to get them quite quickly.

It's quite convenient to have the exception handler showing what kind of exception occurred before anything 'hazardous' is attempted. Displaying, for instance, the (hexadecimal) exception number in a corner of the screen, can save you hours of debugging. :)

<!-- The following code example should be more generalized (e.g. exc_0d_handler, gpfExcHandler renamed to more meaningful names).-->
<source lang="asm">
exc_0d_handler:
    push gs
    mov gs,ZEROBASED_DATA_SELECTOR
    mov word [gs:0xb8000],'D '
    ;; D in the top-left corner means we're handling
    ;;  a GPF exception right ATM.

    ;; your 'normal' handler comes here
    pushad
    push ds
    push es
    mov ax,KERNEL_DATA_SELECTOR
    mov ds,ax
    mov es,ax

    call gpfExcHandler

    pop es
    pop ds
    popad

    mov dword [gs:0xb8000],'  D-'
    ;; the 'D' moved one character to the right, letting
    ;; us know that the exception has been handled properly
    ;; and that normal operations continues.
    pop gs
    iret
</source>

Once you have implemented such a technique, it may be wise to test it, deliberately issuing 'faulty' instructions to see if the correct code is displayed. Having the 'double fault' exception (08) displayed somewhere else on the screen may also be a smart move.

==== What to do if characters cannot be displayed ====
Such things occurs for instance when your GDT or paging tables has been badly configured (e.g. 0xb8000 no longer refers to the video memory). Fortunately enough, the video memory is not your sole communication technique with your kernel:
* you may use the [[PS2 Keyboard|keyboard]] LEDs to report some events (for instance enabling the 'scroll lock' LED when you're a handler and disabling it when you're out).
* you may use the internal [[PC Speaker]] to make Morse-code-like signals reporting early errors. (May be disgraceful if coding late, though.)
* you may use [[VGA Hardware|VGA registers]] to change the background color or the overscan (screen border) color to report the current 'state' of your kernel. (E.g. black = normal operations, yellow = processing interrupt, red = crash condition occurred (just before cli:hlt), blue = processing an exception, etc...)
Refer to [[:Category:VGA|VGA Resources]] to see how you can modify colors. The [[resources]] page should have all the documentation for LED flashing and speaker beeping.
* you may output bytes via the [[serial ports|serial port]]. Most emulators allow you to redirect these characters into a file and unlike the screen the number of characters is not limited. A driver for the serial port is also very easy to implement.

==== Avoiding exception loops ====
So we know when exceptions occur and which exception occurred. That's better but still not especially useful. Your exception handler is likely to become something complex as your kernel will evolve, and you'll discover that exceptions mainly occur ... in exception handlers.

In order to avoid recursive exceptions to occur endlessly, you can easily maintain a 'nested exceptions counter' that will be incremented every time you enter an exception handler and decremented just before you leave that handler. If the counter is above a certain threshold of a few units (3 should give interesting enough results), the kernel will abort trying to solve the exception and enter a 'panic' mode (red background, flashing LED, whatever).

<source lang="c">
int nestexc = 0;

// called by the stub
void gpfExcHandler(void) {
   if (nestexc > MAX_NESTED_EXCEPTIONS) panic();
   nestexc++;

   if (!fix_the_error()) {
     write_an_error_message();
   }
   nestexc--;
   return;
}
</source>

You need to know, of course, that some exceptions are not 'resumable'. If your kernel issued a division by zero, trying to return to the 'div' instruction will only trigger the exception one more time (yeah! altogether, now :). Such loops cannot be solved by the 'nestexc' counter

==== Showing the stack content ====
Much of your program's state (function arguments, return address, local variables) is stored on the [[stack]], especially when using C/C++ code. A complete debugger (like GDB) will inspect the debugging info to give names to the stack content, provide a list of calls, etc. This is a bit complex to do ourselves, but if your kernel can simply ''show'' the content of the stack and if you know ''where'' in the code the process halted, you can already fix quite a lot of bugs by doing the job of the debugger yourself, guessing which stack location holds which variable, where the return addresses are, etc.

The stack content is still in memory. The [[EBP]] value of the erroring process is still in memory, and points to the start of the stack frame for the current function. Everything from this address and up was the current stack. Now, you can use the value in ebp as the source. Just use the following call:

<source lang="asm">
stack_dump:
  push ebp
  mov ebp, esp
  call dump_hex
  pop ebp
  ret ; note that this is not going to work, but it should be here for completion.
</source>

and use <tt>void dump_hex(char *stack)</tt>.

==== Locating the Faulty instruction ====
In most cases, when your exception handler is called, the address of the faulty instruction is somewhere on the stack. The first step here is to print out the address of this instruction.

Once this is done, you (as a human) can inspect the ''linker map'' and find out in which object file the problem was. You can request a map with <tt>ld </tt>''<usual options>''<tt> -Map </tt>''<filename.map>''.

<pre>
.text           0x0000000000005330      0x556 bin/init.o
                0x0000000000005370                kinit_dsp_buffer
                0x0000000000005380                kinit_glocal_tag
                0x0000000000005400                kinit
                0x0000000000005830                kreset
 *fill*         0x0000000000005886        0xa 00
 .text          0x0000000000005890      0x66d bin/kalloc.o
                0x0000000000005ac0                kfree
                0x0000000000005b70                kmRegister
                0x00000000000059b0                kealloc
                0x0000000000005e60                kmFindP
                0x0000000000005dd0                kmFindA
                0x0000000000005d10                kmSetFull
                0x0000000000005890                kalloc
</pre>

Is an example of what a map can look like. If the error address was 0x554f, we can tell from this that the error is somewhere in init.o, and most likely in <tt>kinit()</tt> function (it may still be in some other function if there are some ''static'' functions in the source file). All we know is that the error occurred at offset +21f in the file.

Now, we can use <tt>objdump -drS bin/init.o</tt> to get a look at the disassembled output. Note that this step will work properly only if you had enabled debug information in those separated <tt>.o</tt> files...

<source lang="c">
#ifdef __DEBUG__
  kprint("kernel in debug-mode(%x) press [SHIFT+SPACE] to bypass anykey()\n",
 216:   83 c4 f8                add    $0xfffffff8,%esp
 219:   a1 00 00 00 00          mov    0x0,%eax
                        21a: R_386_32   DbMsk
 21e:   50                      push   %eax
 21f:   68 a0 01 00 00          push   $0x1a0
                        220: R_386_32   .rodata
 224:   e8 fc ff ff ff          call   225 <kinit+0x155>
                        225: R_386_PC32 kprint
         DbMsk);
#endif
</source>

Of course, as I picked up a random address, there's nothing wrong to see at +21f, but I guess you got my point. :)

==== Locating the offending line of source code ====
Once you have found the address of the faulty instruction in the previous step, you can identify the corresponding line of source code by running

<pre>
addr2line -e <your_kernel.elf> <address of faulty instruction>
</pre>

=== Enhanced debugging techniques ===
==== Stack tracing ====
By analyzing the default way to create a stack frame, you can rip off a stack frame at a time, resulting in the call sequence that leads to the fault. For a single bonus point, also extract the arguments and dump them as well. For multiple bonus points, use C++ name mangling, and export the arguments in readable form in the correct type.

Each time a function is called it gets the following head/tail: ([[GCC]] 3.3.2)

<source lang="asm">
push ebp
mov ebp, esp
...
leave
ret
</source>

On the place of the ... the rest of the code is filled in. Now, if you analyze the stack output, it looks something like:

<pre>
0000FFC0 0000FFD0 -> this is the result of a push EBP (which pushes the esp at the start of the previous function)
0000FFC4 001023A5 -> this was the old EIP, which can be looked up in the function table (map file)
0000FFC8 01234567 -> this is an argument
0000FFCC 89ABCDEF -> this is another argument
0000FFD0 0000FFF0 -> this is again another EBP
0000FFD4 00105BC3 -> this is again an EIP
0000FFD8 001023A5 -> this is an argument (could be a function pointer)
0000FFDC 01234567 -> this is another argument
0000FFE0 89ABCDEF -> this is again an argument
0000FFE4 000B8000 -> this is an argument, but not to this function
0000FFE8 FFC00000 -> this is again an argument to a different function
0000FFEC 00010000 -> this is again another argument, but again not to this function.
0000FFF0 0000FFFC -> this is the previous EBP again (note this is in this case the top)
0000FFF4 0010002C -> this is an old EIP
0000FFF8 00000001 -> this is an argument
0000FFFC 00000000 -> this is the EBP at the start of the first function, not necessarily valid!
</pre>

Now, you can traverse along the path of execution. The content of EBP is the old value of EBP, that is, the one of the last stack frame. The value above that is the old instruction pointer (which points inside the current function), and the values above that, up to but not including the value pointed to by the old EBP, are the arguments. Note that the arguments don't have to belong to this function, GCC occasionally saves an add to esp by not popping the values. By then pretending the old EBP is the current EBP, you can unwind another call. Do this until you are fed up by it, you have enough output or the stack ends. If the last one, watch out for not generating a double fault.

If you use C++ name mangling, the arguments are encoded in the function name. If you can read that, you can decode what the value on the stack must be, so you can actually present it to the user in the form of a normal function call with legible arguments and everything. This is the 'crème de la crème' of stack dumping methods, so most aren't expected to do this.

While I program my kernel in C, I actually thought of writing a script that would parse the header files for function declarations, extract the debugging symbols from the compiled kernel image using <tt>objdump</tt>, and write a system map which would provide the types. Forgot it after falling in love with [[Bochs]]' debugger though. Similarly, typemaps for structured types could be created, which would allow the same kind of browsing that GDB or [[Visual Studio]] give you. THIS would be the crème de la crème.

=== Debugging techniques ===
If your function <tt>x()</tt> wreaks havoc only after 1000 calls it may not suffice to put a <tt>panic()</tt> statement inside the functions to see where the functions breaks. You may want to know which call is malignant. To do this, one might use a global or static var to count calls and panic() after an amount to see if it managed to crash. If not, you try twice that amount; if it does crash, you try bisection to find the amount.

<source lang="c">
void scheduler_choose_task() {
    static uint32_t Z=0;
    Z++;
    uint32_t N = 1000;
    if (Z > N) panic();  //find the largest integer N for which it crashes not
    if (in_critical_section()) return;
    ...
}
</source>
...and then check how far does it go:
<source lang="c">
Z++;
uint32_t N = 1000;
                                       //we get here,
if (in_critical_section()) return;
if (Z > N) panic();                    //do we get here to panic() before a crash?
</source>
However as complexity rises or multithreading is involved, it is less probable that a crash would be consistently occurring at the same point, after the same amount of calls every time. Then it would not be possible to find the number of the call to <tt>scheduler_choose_talk()</tt> that crashes it (because that number changes). Debugging needs some imagination; what if you knew, by tracing the program flow with <tt>print(__LINE__)</tt> that <tt>scheduler_choose_task()</tt> crashes only when a call to <tt>fun1()</tt> is in progress? You might use a global var <tt>uint32_t dbg</tt> or an array (<tt>uint32_t dbg[20]</tt>) of various <tt>dbg</tt> vars (which are used only in debugging code which is cleaned after the programmer ceases to debug) in a manner such as:
<source lang="c">
void fun1() {
    dbg[3] = 1;
    ...
    if (x()) { dbg[3]=0; return; }
    ...
    dbg[3] = 0;
}
</source>
...and:
<source lang="c">
void scheduler_choose_task() {
    //  if (dbg[3]==1) panic();         //check here.. a panic saves the day from crashing!
    if (in_critical_section()) return;
    if (dbg[3]==1) panic();             //check here.. it crashes
}
</source>
(Or mix it with a call count, <tt>Z++; if (Z>5 && dbg[3] == 1) panic()</tt>.)

Using the <tt>__LINE__</tt> aids tracing the program flow:
<source lang="c">
print(__LINE__);
</source>
See [[C preprocessor#Uses for debugging|Uses for debugging]] for more info.

== External assistance ==
So far, we assumed that the kernel was containing all the information required for debugging (like symbols names, etc). In production-stage, however, this information, as well as most of the 'debugging prints' have usually been stripped out of the final binary object.

Still, one could imagine a kernel that would be equipped with serial-line communication code and connected to another computer that would have all the 'removed' information (like the symbols map, or the debugging-info featured intermediate binaries).

In case of a panic, the kernel could for instance send the value of <tt>eip</tt> over the serial line and expect the helper PC to reply with the function name and line number corresponding to that address (or dump it on the helper PC's screen, that's a matter of choice :) ).

=== Debugging interface ===
''Now we have plenty of information about what was wrong... can we ask for more? What do Mobius' debugging shell and Clicker's information panels tell us...''

''Does anybody else know of an OS that allows the hacker to interactively probe the system state when crashes occur ?''

:''Yes. Guess. Correct: AmigaOS. ;-) It offered a mode that allowed debugging over serial line even after the system went into Guru Meditation. That was possible because AmigaOS enjoyed a 256 / 512 kbyte ROM image that could not get corrupted.'' - [[User:Solar|MartinBaute]]

''Unix systems traditionally write their state to /dev/core for offline guru meditation''

== See Also ==
=== Articles ===
* [[How Do I Use A Debugger With My OS]]

[[Category:Troubleshooting]]
[[Category:Debugging]]
[[de:Debugging]]
````

<a id="w001541"></a>

## W001541 — Pascal

Bron-ID: W001541
Namespace: Main (0)
Revisie: 30257 | Datum: 2026-03-16T16:00:38Z
Bron: https://wiki.osdev.org/Pascal
Revisie-URL: https://wiki.osdev.org/index.php?title=Pascal&oldid=30257

````text
== Historical note ==
The original Standard Pascal was in many ways a different language from the Object Pascal that most people today are familiar with, being much simpler but also more limited.

Dr. Wirth himself did not intend Pascal for systems programming, and his ongoing language evolution were the languages Modula-2 and Oberon. Both languages are related to Pascal in a similar way as C++, C# and Java are related to C. Oberon-2 supports all concepts of object-oriented programming. In contrast to the C language family, in the course from Pascal to Oberon the language definition got a lot more compact, but the language itself got more powerful.

The successors of Pascal were developed to address the weaknesses of Pascal in this regard. However, with the widespread adoption of the Object Pascal extensions (e.g.. unit, bitwise operators), many of these weaknesses (most specifically the lack of support for separate compilation) were eliminated. The reputation of Pascal as a toy language has unfairly persisted in many places however.

=== Past uses in OS development ===
Pascal was used in early Apple Macs as implementation language.

Oberon as a successor of Pascal has been used extensively to develop and research Operating Systems (see [http://www.oberon.ethz.ch/ Native Oberon] and [http://bluebottle.ethz.ch BlueBottle]).

== Commonly used tools ==
The most popular pascal compilers today seems to be Delphi and FreePascal Compiler, and to a smaller extent Turbo Pascal.

There is a commercial (proprietary license, apparently) operating system being developed in Delphi, [http://www.tattsoft.com/index.php/products/9-operating-systems/10-classios-an-object-pascal-operating-system.html ClassiOS], a Windows clone, (formerly [http://www.tattsoft.com/index.php/products/petrosr.html PetrOS®], developed with an internal Pascal compiler) by Peter Tattam (best known for his TCP/IP stack for DOS, the Trumpet line of products). He customized Delphi to [http://web.archive.org/web/20110219051243/http://petertattam.com/?p=19 produce Windows device drivers] too.

Anyway, FPC (FreePascal Compiler) seems more suited to develop a portable operating system as it's highly configurable and generates code to a great number of platforms.

== Interfacing Pascal with Assembler ==
Just like when Doing a kernel in C++, Pascal compilers mangle functions name to make them convey more information (such as arguments and return types). That means if you just write

<syntaxhighlight lang="pascal">
unit KernelMain;

interface

implementation

procedure kernel_main;
begin
    ...
end;

end.
</syntaxhighlight>

You may end up with "THREADVARLIST_P$KERNEL_MAIN" rather than just "kernel_main" as a function name. If you're using FreePascal, the tool <tt>objdump</tt> can show you the symbol table of the .o file generated by the compiler, which will give you the "real" name of the function.

Alternatively, you could use compiler extra statements to enforce a "public name" to your function:

<syntaxhighlight lang="pascal">
unit KernelMain;

interface

implementation

procedure kernel_main; [public, alias: 'KERNEL_MAIN'];
begin
    ...
end;

end.
</syntaxhighlight>

Finally, simply declaring a program like you would when writing Pascal code for any normal platform will create a main routine named PASCALMAIN.

<syntaxhighlight lang="pascal">
program Kernel;

uses Console,Stuff,Etc;

var
 stuff: type;

begin
{Your kernel here.}
end.
</syntaxhighlight>

Note, too, that C and PASCAL doesn't share the same calling convention. Most notably, arguments in PASCAL are pushed from left to right while C push them from right to left. If this gets you in trouble, you can use cdecl modifier to force the compiler considering that your PASCAL procedure works like a C function (that should mainly be useful to interface pascal code with C code). Moreover, in PASCAL, the callee function is responsible from [[Stack#Stack example on the X86 architecture|stack cleaning]], while this is typically the job of the caller in C/C++ environment.

== See Also ==

* [[Pascal Bare Bones]]
* [[BOOTBOOT]] loader has an example 64 bit higher half kernel in Pascal

[[Category:Languages]]
[[Category:Pascal]]
````

<a id="w001542"></a>

## W001542 — ELF

Bron-ID: W001542
Namespace: Main (0)
Revisie: 30569 | Datum: 2026-05-10T03:28:40Z
Bron: https://wiki.osdev.org/ELF
Revisie-URL: https://wiki.osdev.org/index.php?title=ELF&oldid=30569

````text
{{File formats}}

ELF (Executable and Linkable Format) was designed by Unix System Laboratories while working with Sun Microsystems on SVR4 (UNIX System V Release 4.0). Consequently, ELF first appeared in Solaris 2.0 (aka SunOS 5.0), which is based on SVR4. The format is specified in the [[System V ABI]].

A very versatile file format, it was later picked up by many other operating systems for use as both executable files and as shared library files. It does distinguish between TEXT, DATA and BSS.

Today, ELF is considered the standard format on Unix-alike systems. While it has some drawbacks (e.g., using up one of the scarce general purpose registers of the IA-32 when using position-independent code), it is well supported and documented.

==File Structure==

ELF is a format for storing many program types (see ELF Header table) on the disk, created as a result of compiling and linking. An ELF file might independently contain sections or segments. For an executable program, an ELF header and a segment are the bare minimum, while sections are optional, though it's common for an executable to have a ".text" section for the code and ".data" section for initialized data. Libraries don't have segments, but only sections because they are used for linking purposes. Sections and segments are described by their respective headers that contain information about their sizes, required alignment, etc.

Note that depending on whether your file is a linkable or an executable file, the headers in the ELF file won't be the same:
process.o, result of gcc -c process.c $SOME_FLAGS

<pre>
C32/kernel/bin/.process.o
architecture: i386, flags 0x00000011:
HAS_RELOC, HAS_SYMS
start address 0x00000000

Sections:
Idx Name          Size      VMA       LMA       File off  Algn
  0 .text         00000333  00000000  00000000  00000040  2**4
                  CONTENTS, ALLOC, LOAD, RELOC, READONLY, CODE
  1 .data         00000050  00000000  00000000  00000380  2**5
                  CONTENTS, ALLOC, LOAD, DATA
  2 .bss          00000000  00000000  00000000  000003d0  2**2
                  ALLOC
  3 .note         00000014  00000000  00000000  000003d0  2**0
                  CONTENTS, READONLY
  4 .stab         000020e8  00000000  00000000  000003e4  2**2
                  CONTENTS, RELOC, READONLY, DEBUGGING
  5 .stabstr      00008f17  00000000  00000000  000024cc  2**0
                  CONTENTS, READONLY, DEBUGGING
  6 .rodata       000001e4  00000000  00000000  0000b400  2**5
                  CONTENTS, ALLOC, LOAD, READONLY, DATA
  7 .comment      00000023  00000000  00000000  0000b5e4  2**0
                  CONTENTS, READONLY
</pre>

The 'flags' will tell you what's actually available in the ELF file. Here, we have [[Symbol_Table|symbol tables]] and relocation: all that we need to link the file against another, but virtually no information about how to load the file in memory (even if that could be guessed). We don't have the program entry point, for instance, and we have a sections table rather than a program header.

{| class="wikitable"
! Section
! Description
|-
| <tt>.text</tt>
| Where the actual code is stored. <code>objdump -drS .process.o</code> will show you that
|-
| <tt>.data</tt>
| Where global tables, variables, etc. live. <code>objdump -s -j .data .process.o</code> will hexdump it.
|-
| <tt>.bss</tt>
| This is where uninitialized variables are located, this section is not stored in the disk, as it is expected to be created by the loader and fill it with zeros in memory.
|-
| <tt>.rodata</tt>
| "Read-only data" section. This is where the strings go. <code>objdump -s -j .rodata .process.o</code> will hexdump it. Note that depending on the compiler, more sections like this may exist.
|-
| <tt>.comment</tt> and <tt>.note</tt>
| Comments put there by the compiler/linker toolchain.
|-
| <tt>.stab</tt> and <tt>.stabstr</tt>
| Debugging symbols and similar information.
|}

<tt>/bin/bash</tt>, a real executable file

<pre>
/bin/bash:     file format elf32-i386
/bin/bash
architecture: i386, flags 0x00000112:
EXEC_P, HAS_SYMS, D_PAGED
start address 0x08056c40

Program Header:
    PHDR off    0x00000034 vaddr 0x08048034 paddr 0x08048034 align 2**2
         filesz 0x000000e0 memsz 0x000000e0 flags r-x
</pre>
The program header itself... taking 224 bytes, and starting at offset 0x34 in the file
<pre>
  INTERP off    0x00000114 vaddr 0x08048114 paddr 0x08048114 align 2**0
         filesz 0x00000013 memsz 0x00000013 flags r--
</pre>
The program that should be used to 'execute' the binary. Here, it reads as '/lib/ld-linux.so.2', which means some dynamic libraries linking will be required before we run the program.
<pre>
    LOAD off    0x00000000 vaddr 0x08048000 paddr 0x08048000 align 2**12
         filesz 0x0007411c memsz 0x0007411c flags r-x
</pre>
Now we're requested to read 7411c bytes, starting at file's start (?) and being 7411c bytes large (that's virtually the whole file!), which will be read-only but executable. They'll be to appear starting at virtual address 0x08048000 for the program to work properly.
<pre>
    LOAD off    0x00074120 vaddr 0x080bd120 paddr 0x080bd120 align 2**12
         filesz 0x000022ac memsz 0x000082d0 flags rw-
</pre>
More bits to load, (likely to be .data section). Notice that the 'filesize' and 'memsize' differ, which means the .bss section will actually be allocated through this statement, but left as zeroes while 'real' data only occupy first 0x22ac bytes starting at virtual address 0x80bd120.
<pre>
 DYNAMIC off    0x00075f4c vaddr 0x080bef4c paddr 0x080bef4c align 2**2
         filesz 0x000000e8 memsz 0x000000e8 flags rw-
</pre>
The dynamic sections are used to store information used in the dynamic linking process, such as required libraries and relocation entries.
<pre>
    NOTE off    0x00000128 vaddr 0x08048128 paddr 0x08048128 align 2**2
         filesz 0x00000020 memsz 0x00000020 flags r--
</pre>
NOTE sections contain information left by either the programmer or the linker, for most programs linked using the GNU 'ld' linker it just says 'GNU'
<pre>
EH_FRAME off    0x000740f0 vaddr 0x080bc0f0 paddr 0x080bc0f0 align 2**2
         filesz 0x0000002c memsz 0x0000002c flags r--
</pre>
That's for Exception Handler information, in case we should link against some C++ binaries at execution (Needs citing).
<pre>
/bin/bash, loaded (as in /proc/xxxx/maps)

08048000-080bd000 r-xp 00000000 03:06 30574      /bin/bash
080bd000-080c0000 rw-p 00074000 03:06 30574      /bin/bash
080c0000-08103000 rwxp 00000000 00:00 0
40000000-40014000 r-xp 00000000 03:06 27304      /lib/ld-2.3.2.so
40014000-40015000 rw-p 00013000 03:06 27304      /lib/ld-2.3.2.so
</pre>

We can recognize our 'code bits' and 'data bits', by stating that the second one should be loaded at 0x080bd*120* and that it starts in file at 0x00074*120*, we actually preserved page-to-disk blocks mapping (e.g. if page 0x80bc000 is missing, just fetch file blocks from 0x75000). That means, however, that a part of the code is mapped twice, but with different permissions. I suggest you do give them different physical pages too if you don't want to end up with modifiable code.

==Loading ELF Binaries==

[[Image:Elfdiagram.png|frame|Executable image and elf binary can being mapped onto each other]]
The ELF header contains all of the relevant information required to load an ELF executable. The format of this header is described in the [https://refspecs.linuxfoundation.org/elf/elf.pdf ELF Specification]. The most relevant sections for this purpose are 1.1 to 1.4 and 2.1 to 2.7. Instructions on loading an executable are contained within section 2.7.

The following is a rough outline of the steps that an ELF executable loader must perform:

* Verify that the file starts with the ELF magic number (4 bytes) as described in figure 1-4 (and subsequent table) on page 11 in the ELF specification.
* Read the ELF Header. The ELF header is always located at the very beginning of an ELF file. The ELF header contains information about how the rest of the file is laid out. An executable loader is only concerned with the program headers.
* Read the ELF executable's program headers. These specify where in the file the program segments are located, and where they need to be loaded into memory.
* Parse the program headers to determine the number of program segments that must be loaded. Each program header has an associated type, as described in Figure 2-2 of the ELF specification. Only headers with a type of <tt>PT_LOAD</tt> describe a loadable segment.
* Load each of the loadable segments. This is performed as follows:
** Allocate virtual memory for each segment, at the address specified by the <tt>p_vaddr</tt> member in the program header. The size of the segment in memory is specified by the <tt>p_memsz</tt> member.
** Copy the segment data from the file offset specified by the <tt>p_offset</tt> member to the virtual memory address specified by the <tt>p_vaddr</tt> member. The size of the segment in the file is contained in the <tt>p_filesz</tt> member. This can be zero.
** The <tt>p_memsz</tt> member specifies the size the segment occupies in memory. This can be zero. If the <tt>p_filesz</tt> and <tt>p_memsz</tt> members differ, this indicates that the segment is padded with zeros. All bytes in memory between the ending offset of the file size, and the segment's virtual memory size are to be cleared with zeros.
* Read the executable's entry point from the ELF header.
* Jump to the executable's entry point in the newly loaded memory.

==Relocation==

Relocation becomes handy when you need to load, for example, modules or drivers. It's possible to use the "-r" option to ld to permit you to have multiple object files linked into one big one, which means easier coding and faster testing.

The basic outline of things you need to do for relocation:
# Check the object file header (it has to be ELF, not PE, for example)
# Get a load address (eg. all drivers start at 0xA0000000, need some method of keeping track of driver locations)
# Allocate enough space for all program sections (ST_PROGBITS)
# Copy from the image in RAM to the allocated space
# Go through all sections resolving external references against the kernel symbol table
# If all succeeded, you can use the "e_entry" field of the header as the offset from the load address to call the entry point (if one was specified), or do a symbol lookup, or just return a success error code.

Once you can relocate ELF objects you'll be able to have drivers loaded when needed instead of at startup - which is always a Good Thing (tm).

== Tables ==
=== ELF Header ===
The ELF header is always found at the start of the file.
{| class="wikitable"
|-
! Position (32 bit)
! Position (64 bit)
! Value
|-
| 0-3
| 0-3
| Magic number - 0x7F, then 'ELF' in ASCII
|-
| 4
| 4
| 1 = 32 bit, 2 = 64 bit
|-
| 5
| 5
| 1 = little endian, 2 = big endian
|-
| 6
| 6
| ELF header version
|- 
| 7
| 7
| OS ABI - usually 0 for System V
|-
| 8-15
| 8-15
| Unused/padding
|-
| 16-17
| 16-17
| Type (1 = relocatable, 2 = executable, 3 = shared, 4 = core)
|-
| 18-19
| 18-19
| Instruction set - see table below
|-
| 20-23
| 20-23
| ELF Version (currently 1)
|- 
| 24-27
| 24-31
| Program entry offset
|-
| 28-31
| 32-39
| Program header table offset
|-
| 32-35
| 40-47
| Section header table offset
|-
| 36-39
| 48-51
| Flags - architecture dependent; see note below
|-
| 40-41
| 52-53
| ELF Header size
|-
| 42-43
| 54-55
| Size of an entry in the program header table
|-
| 44-45
| 56-57
| Number of entries in the program header table
|-
| 46-47
| 58-59
| Size of an entry in the section header table
|-
| 48-49
| 60-61
| Number of entries in the section header table
|-
| 50-51
| 62-63
| Section index to the section header string table
|}

The flags entry can probably be ignored for x86 ELFs, as no flags are actually defined.

=== Program header ===
This is an array of N (given in the main header) entries in the following format. Make sure to use the correct version depending on whether the file is 32 bit or 64 bit as the tables are quite different.

{| class="wikitable"
|+ 32 bit version
! Position
! Value
|-
| 0-3
| Type of segment (see below)
|-
| 4-7
| The offset in the file that the data for this segment can be found (p_offset)
|-
| 8-11
| Where you should start to put this segment in virtual memory (p_vaddr)
|-
| 12-15
| Reserved for segment's physical address (p_paddr)
|-
| 16-19
| Size of the segment in the file (p_filesz)
|-
| 20-23
| Size of the segment in memory (p_memsz, at least as big as p_filesz)
|-
| 24-27
| Flags (see below)
|-
| 28-31
| The required alignment for this section (usually a power of 2)
|}

{| class="wikitable"
|+ 64 bit version
! Position
! Value
|-
| 0-3
| Type of segment (see below)
|-
| 4-7
| Flags (see below)
|-
| 8-15
| The offset in the file that the data for this segment can be found (p_offset)
|-
| 16-23
| Where you should start to put this segment in virtual memory (p_vaddr)
|-
| 24-31
| Reserved for segment's physical address (p_paddr)
|-
| 32-39
| Size of the segment in the file (p_filesz)
|-
| 40-47
| Size of the segment in memory (p_memsz, at least as big as p_filesz)
|-
| 48-55
| The required alignment for this section (usually a power of 2)
|}

Segment types: 0 = null - ignore the entry; 1 = load - clear p_memsz bytes at p_vaddr to 0, then copy p_filesz bytes from p_offset to p_vaddr; 2 = dynamic - requires dynamic linking; 3 = interp - contains a file path to an executable to use as an interpreter for the following segment;  4 = note section. There are more values, but mostly contain architecture/environment specific information, which is probably not required for the majority of ELF files.

Flags: 1 = executable, 2 = writable, 4 = readable.

=== Instruction Set Architectures ===
{| class="wikitable"
|-
! Architecture
! Value
|-
| No Specific
| <tt>0x00</tt>
|-
| [[:Category:Sparc | Sparc]]
| <tt>0x02</tt>
|-
| '''[[x86]]'''
| <tt>0x03</tt>
|-
| [[:Category:MIPS | MIPS]]
| <tt>0x08</tt>
|-
| [[PA-RISC]]
| <tt>0x0F</tt>
|-
| [[PowerPC]], 32-bit
| <tt>0x14</tt>
|-
| [[PowerPC]], 64-bit
| <tt>0x15</tt>
|-
| [[S390]]
| <tt>0x16</tt>
|-
| '''[[:Category:ARM | ARM]]'''
| <tt>0x28</tt>
|-
| [[Alpha]]
| <tt>0x29</tt>
|-
| [https://en.wikipedia.org/wiki/SuperH SuperH]
| <tt>0x2A</tt>
|-
| [[IA-64]]
| <tt>0x32</tt>
|-
| '''[[x86-64]]'''
| <tt>0x3E</tt>
|-
| '''[[:Category:ARM | AArch64]]'''
| <tt>0xB7</tt>
|-
| [[RISC-V]]
| <tt>0xF3</tt>
|}
The most common architectures are in bold.

=== Dynamic Linking ===
{{Main|Dynamic Linker}}

Dynamic Linking is when the OS gives a program shared libraries if it needs them. Meaning, the libraries are found in the system and then "bind" to the program that needs them while the program is running, versus static linking, which links the libraries '''before''' the program is run. The main advantages are that programs take up less memory, and are smaller in file size. The main disadvantage, however, is that the program becomes less portable because the program depends on many different shared libraries.

In order to implement this, you need to have proper scheduling in place, a library, and a program to use that library.
You can create a library with GCC:

<pre>
myos-gcc -c -fPIC -o oneobject.o oneobject.c
myos-gcc -c -fPIC -o anotherobject.o anotherobject.c
myos-gcc -shared -fPIC -Wl,-soname,nameofmylib oneobject.o anotherobject.o -o mylib.so
</pre>

This library should be treated as a file, which is loaded when the OS detects its attempted usage. You will need to implement this "[[Dynamic Linker]]" into a certain classification of code such as in your memory management or your task management section. When the ELF program is run, the system should attach the shared object data to a malloc() region of memory, where the function calls to the libraries redirect to that malloc() region of memory. Once the program is finished, the region can be given up back to the OS with a call to free().

That should be a good starting point to writing a dynamic linker.

== See Also ==
=== Articles ===
* [[System V ABI]]
* [[Symbol Table]]
* [[Modular Kernel]]
* [[DWARF]]

=== External Links ===
* [https://web.archive.org/web/20250628070854/https://www.skyfree.org/linux/references/ELF_Format.pdf The ELF file format] in detail
* [http://docs.oracle.com/cd/E23824_01/html/819-0690/chapter6-46512.html#scrolltoc ELF Format Specifications] Detailed and up-to-date ELF information (including SPARC in depth) by Oracle.
* [http://www.sco.com/developers/gabi/latest/contents.html System V ABI] about ELF 
* [http://www.linuxfoundation.org/en/Specifications LSB specifications]<br />See (generic or platform-specific) 'Core' specifications for additional ELF information.
<!--
Link is dead.
*ftp://tsx.mit.edu/pub/linux/packages/GCC/ELF.doc.tar.gz
-->
* [http://en.wikipedia.org/wiki/Executable_and_Linkable_Format Executable and Linkable Format on Wikipedia],which contains a detail of elf references
* [https://uclibc.org/docs/elf-64-gen.pdf The ELF file format(64-bit)] ELF 64-Bit, General extension to ELF32.
* [http://www.x86-64.org/documentation/abi.pdf x86-64 ABI] Documented x86-64 specific extensions with ELF64.
* [http://www.robinhoksbergen.com/papers/howto_elf.html Manually Creating an ELF Executable] (dead, [https://web.archive.org/web/20140130143820/http://www.robinhoksbergen.com/papers/howto_elf.html link from archive.org]) Detailed guide on how to create ELF binaries from scratch.
* [https://www.youtube.com/playlist?list=PLZCIHSjpQ12woLj0sjsnqDH8yVuXwTy3p Handmade Linux x86 executables] Youtube playlist about Linux x86 executables, explains ELF binary structure
[[Category:ABI]]
[[Category:Executable Formats]]
[[Category:Object Files]]
[[Category:Standards]]
[[de:Executable and Linking Format]]
````

<a id="w001543"></a>

## W001543 — File:Elfdiagram.png

Bron-ID: W001543
Namespace: File (6)
Revisie: 1955 | Datum: 2006-12-08T07:28:59Z
Bron: https://wiki.osdev.org/File:Elfdiagram.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AElfdiagram.png&oldid=1955

````text

````

<a id="w001544"></a>

## W001544 — GDB

Bron-ID: W001544
Namespace: Main (0)
Revisie: 28511 | Datum: 2023-12-13T07:13:30Z
Bron: https://wiki.osdev.org/GDB
Revisie-URL: https://wiki.osdev.org/index.php?title=GDB&oldid=28511

````text
'''GDB''' has become the standard debugger on Linux and other free Unices. It is a source-level debugger rather than a machine-level one such as the Bochs debugger; this could be good or bad, depending on your viewpoint.

While debugging system code is not its intended use, it does a fine job of it; it can be used directly with some emulators, without modifying your code at all, or it can be used over a serial line. The latter option involves implementing a Remote Serial Protocol stub in your operating system.

==Remote Debugging==
We are interested in the remote debugging facility of GDB. From the [http://sourceware.org/gdb/current/onlinedocs/gdb/Remote-Debugging.html GDB manual], “If you are trying to debug a program running on a machine that cannot run GDB in the usual way, it is often useful to use remote debugging. For example, you might use remote debugging on an operating system kernel, or on a small system which does not have a general purpose operating system powerful enough to run a full-featured debugger.”

For remote debugging, a remote stub should be implemented in the remote program that needs to be debugged. That means the kernel should contain the remote stub to talk to the host GDB during the debug session. This requires a kernel source change and it is a must if you are running your kernel on a test(real) machine and debugging from another machine. If you are using an emulator (Bochs or QEMU), you can use the GDB stub compiled into the emulator.

===Implementing GDB Stub===
To debug (using GDB) a kernel running on a real machine, the kernel needs to contain a GDB stub. For the i386 platform, GDB source includes a reference implementation of gdb-stub.c. It requires the following three functions from the kernel to read/write from a serial port and to set up the exception handlers.

<source lang="C">
   getDebugChar(); // to read a character from serial port
   putDebugChar(int); // to write a character to serial port
   exceptionHandler(int exception_number, void *exception_address); // to set exception handler
</source>

Implement the above functions in your kernel, include the gdb-stub.c in the compilation, and also call the following functions during kernel start to connect to the gdb host.

<source lang="C">
   InitSerialPort(sys_gdb_port, 9600, UART_DATA_BIT_8, UART_PARITY_NONE, UART_STOP_BIT_1);   /*set up the serial port*/
   set_debug_traps();   /*setup exception handlers*/
   kprintf("Waiting for GDB(0x%X) : ", sys_gdb_port );
   __asm__("int3");   /*break point exception to sync with GDB host*/
</source>

Now start your test machine and your kernel will wait for a GDB host connection.

===Using Emulator Stubs===
'''QEMU''' - GDB can be connected to [[QEMU]] by simply starting qemu with the <tt>-s -S</tt> command line switches. 

'''Bochs''' - For GDB to be able to interface with [[Bochs]], Bochs will need to be configured with the <tt>--enable-gdb-stub</tt> argument.
The <tt>bochssrc</tt> (or whatever configuration script is being used) will need to have the <tt>gdbstub</tt> line set to something like ''gdbstub: enabled=1, port=1234, text_base=0, data_base=0, bss_base=0''.

==Starting GDB==
After an emulator/kernel is configured it will then wait for a connection from GDB. GDB on the host machine can be started like this:
 $ gdb YOUR-KERNEL
 .
 .
 .
 (gdb) target remote :1234
 Remote debugging using :1234
 0x0000fff0 in ?? ()
 (gdb)

If you are debugging a kernel running on a real machine then use ''target remote /dev/tty2'' instead of the network port.

If it prompts you after the "<tt>target remote</tt>" line asking whether to kill a program that is already being debugged answer "<tt>n</tt>".

If you are using a 64-bit kernel, you may need to set up the address size using set command.

===gdbinit files===
Put commonly-used commands in a .gdbinit file in your working directory.  For example:

  file kernel.bin
  target remote localhost:1234

This will connect to GDB-stub whenever you start GDB and will load symbols from kernel.bin.

===GDB in Emacs===

Emacs is well integrated with GDB.  Try using <tt>M-x gdb</tt> and when it asks you for command-line arguments to GDB, use <tt>--annotate=3</tt>.  This will allow Emacs to follow along as you place breakpoints and single-step through your kernel.

===Virtual Serial Ports===
If you have gdbstub inside your kernel and run your kernel inside an emulator, you can use a [[wikipedia:COM port redirector|COM Port redirector]] to create a virtual serial port. After creating a virtual serial port, one port in the pair should be used for the emulator; the other one should specified in gdb for remote debugging.

[http://com0com.sourceforge.net/ com0com] can be used in Windows and a pseudo terminal might be used in Linux to create virtual serial port pairs.

== See Also ==
=== External Links ===
* [http://stackoverflow.com/questions/1471226/most-tricky-useful-commands-for-gdb-debugger StackOverflow: Most tricky / useful commands for GDB debugger]

[[Category:Debugging]]
[[Category:Tools]]
[[de:GDB]]
````

<a id="w001545"></a>

## W001545 — Interrupt Descriptor Table

Bron-ID: W001545
Namespace: Main (0)
Revisie: 30250 | Datum: 2026-03-16T15:50:39Z
Bron: https://wiki.osdev.org/Interrupt_Descriptor_Table
Revisie-URL: https://wiki.osdev.org/index.php?title=Interrupt_Descriptor_Table&oldid=30250

````text
The '''Interrupt Descriptor Table''' ('''IDT''') is a binary data structure specific to the [[IA32_Architecture_Family|IA-32]] and [[X86-64|x86-64]] architectures. It is the [[Protected Mode]] and Long Mode counterpart to the [[Real Mode]] Interrupt Vector Table ([[IVT]]) telling the CPU where the [[Interrupt Service Routines]] (ISR) are located (one per interrupt vector). It is similar to the [[Global Descriptor Table]] in structure.

The IDT entries are called gates. It can contain Interrupt Gates, Task Gates and Trap Gates.

Before you implement the IDT, make sure you have a working GDT.

== IDTR ==

The location of the '''IDT''' is kept in the '''IDTR''' ('''IDT''' register). This is loaded using the '''LIDT''' assembly instruction, whose argument is a pointer to an '''IDT Descriptor''' structure:

{| class="wikitable"
|+IDT Descriptor (IDTR):
!style="width: 66%; text-align: left;" |79 (64-bit Mode)<br>47 (32-bit Mode)&nbsp;&nbsp;&nbsp;<span style="float: right;">16</span>
!style="width: 34%; text-align: left; vertical-align: bottom;" |15&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|-
|'''Offset'''<br>63 (64-bit Mode)<br>31 (32-bit Mode)&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|'''Size'''<br><br>15&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|}

* '''Size''': One less than the size of the '''IDT''' in bytes.
* '''Offset''': The linear address of the '''Interrupt Descriptor Table''' (not the physical address, paging applies).

Note that the amount of data loaded by '''LIDT''' differs in 32-bit and 64-bit modes, '''Offset''' is 4 bytes long in 32-bit mode and 8 bytes long in 64-bit mode.

This is similar to the '''[[GDT]]''', except:
* The first entry (at zero offset) is used in the '''IDT'''.
* There are 256 interrupt vectors (0..255), so the '''IDT''' should have 256 entries, each entry corresponding to a specific interrupt vector.
* Although the '''IDT''' can contain more than 256 entries, they are ignored.
* Although the '''IDT''' can contain less than 256 entries, any entries that are not present (due to this or other reasons) will generate a '''[[Exceptions#General Protection Fault|General Protection Fault]]''' when an attempt to access them is made. Ideally the '''IDT''' should contain enough entries so that this fault (which is itself an interrupt vector) can be handled.

For more information, see '''Section 2.4.3: IDTR Interrupt Descriptor Table Register''' and '''Figure 2-6: Memory Management Registers''' of the Intel Software Developer Manual, Volume 3-A.

== Structure on IA-32 ==

=== Table ===

On 32-bit processors, the entries in the '''IDT''' are 8 bytes long and form a table like this:

{|class="wikitable"
|+Interrupt Descriptor Table (32-bit)
! Address !! Content
|-
| IDTR Offset + 0 || Entry 0
|-
| IDTR Offset + 8 || Entry 1
|-
| IDTR Offset + 16 || Entry 2
|- style="text-align: center;"
| '''...''' || '''...'''
|-
| IDTR Offset + 2040 || Entry 255
|}

The corresponding entry for a given '''Interrupt Vector''' is pointed to in memory by scaling the vector by 8 and adding it to the value in the '''Offset''' field of the '''IDTR'''.

=== Gate Descriptor ===

Each entry in the table has a complex structure:

{| class="wikitable"
|+Gate Descriptor (32-bit):
!style="width: 50%; text-align: left;"|63&nbsp;&nbsp;&nbsp;<span style="float: right;">48</span>
!style="width: 3.125%"|47
!style="width: 6.25%; text-align: left;"|46&nbsp;&nbsp;&nbsp;<span style="float: right;">45</span>
!style="width: 3.125%"|44
!style="width: 12.5%; text-align: left;"|43&nbsp;&nbsp;&nbsp;<span style="float: right;">40</span>
!style="width: 25%; text-align: left;"|39&nbsp;&nbsp;&nbsp;<span style="float: right;">32</span>
|-
|'''Offset'''<br>31&nbsp;&nbsp;&nbsp;<span style="float: right;">16</span>
|style="text-align: center; vertical-align: top;"|'''P'''
|'''DPL'''<br>1&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|style="text-align: center; vertical-align: top;"|0
|'''Gate Type'''<br>3&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|style="vertical-align:top"|Reserved
|-
!style="text-align: left;" |31&nbsp;&nbsp;&nbsp;<span style="float: right;">16</span>
!style="text-align: left;" colspan="5" |15&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|-
|'''Segment Selector'''<br>15&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|colspan="5" |'''Offset'''<br>15&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|}

* '''Offset:''' A 32-bit value, split in two parts. It represents the address of the entry point of the '''[[Interrupt Service Routines|Interrupt Service Routine]]'''.
* '''Selector:''' A '''[[Segment Selector]]''' with multiple fields which must point to a valid code segment in your '''[[GDT]]'''.
* '''Gate Type:''' A 4-bit value which defines the type of gate this '''Interrupt Descriptor''' represents. There are five valid type values:
** '''0b0101''' or '''0x5''': Task Gate, note that in this case, the '''Offset''' value is unused and should be set to zero.
** '''0b0110''' or '''0x6''': 16-bit Interrupt Gate
** '''0b0111''' or '''0x7''': 16-bit Trap Gate
** '''0b1110''' or '''0xE''': 32-bit Interrupt Gate
** '''0b1111''' or '''0xF''': 32-bit Trap Gate
* '''DPL:''' A 2-bit value which defines the '''[[Security#Rings|CPU Privilege Levels]]''' which are allowed to access this interrupt via the '''INT''' instruction. Hardware interrupts ignore this mechanism.
* '''P:''' Present bit. Must be set ('''1''') for the descriptor to be valid.

For more information, see '''Section 7.11: IDT Descriptors''' and '''Figure 7-2: IDT Gate Descriptors''' of the [https://www.intel.com/content/www/us/en/architecture-and-technology/64-ia-32-architectures-software-developer-vol-3a-part-1-manual.html Intel Software Developer Manual, Volume 3-A].

=== Example Code ===

C Struct:

<syntaxhighlight lang="c">
struct InterruptDescriptor32 {
   uint16_t offset_1;        // offset bits 0..15
   uint16_t selector;        // a code segment selector in GDT or LDT
   uint8_t  zero;            // unused, set to 0
   uint8_t  type_attributes; // gate type, dpl, and p fields
   uint16_t offset_2;        // offset bits 16..31
};
</syntaxhighlight>

Example ''type_attributes'' values that people are likely to use (assuming DPL is 0):
* 32-bit '''Interrupt Gate: 0x8E''' (p=1, dpl=0b00, type=0b1110 => type_attributes=0b1000_1110='''0x8E''')
* 32-bit '''Trap Gate: 0x8F''' (p=1, dpl=0b00, type=0b1111 => type_attributes=1000_1111b='''0x8F''')
* '''Task Gate: 0x85''' (p=1, dpl=0b00, type=0b0101 => type_attributes=0b1000_0101='''0x85''')

== Structure on x86-64 ==

=== Table ===

On 64-bit processors, the entries in the '''IDT''' are 16 bytes long and form a table like this:

{|class="wikitable"
|+Interrupt Descriptor Table (64-bit)
! Address !! Content
|-
| IDTR Offset + 0 || Entry 0
|-
| IDTR Offset + 16 || Entry 1
|-
| IDTR Offset + 32 || Entry 2
|- style="text-align: center;"
| '''...''' || '''...'''
|-
| IDTR Offset + 4080 || Entry 255
|}

The corresponding entry for a given '''Interrupt Vector''' is pointed to in memory by scaling the vector by 16 and adding it to the value in the '''Offset''' field of the '''IDTR'''.

=== Gate Descriptor ===

Each entry in the table has a complex structure:

{| class="wikitable"
|+Gate Descriptor (64-bit):
!colspan="7" style="text-align: left;"|127&nbsp;&nbsp;&nbsp;<span style="float: right;">96</span>
|-
|colspan="7" |Reserved
|-
!colspan="7" style="text-align: left;"|95&nbsp;&nbsp;&nbsp;<span style="float: right;">64</span>
|-
|colspan="7" |'''Offset'''<br>63&nbsp;&nbsp;&nbsp;<span style="float: right;">32</span>
|-
!style="width: 50%; text-align: left;"|63&nbsp;&nbsp;&nbsp;<span style="float: right;">48</span>
!style="width: 3.125%"|47
!style="width: 6.25%; text-align: left;"|46&nbsp;&nbsp;&nbsp;<span style="float: right;">45</span>
!style="width: 3.125%"|44
!style="width: 12.5%; text-align: left;"|43&nbsp;&nbsp;&nbsp;<span style="float: right;">40</span>
!style="width: 15.625%; text-align: left;"|39&nbsp;&nbsp;&nbsp;<span style="float: right;">35</span>
!style="width: 9.375%; text-align: left;"|34&nbsp;&nbsp;&nbsp;<span style="float: right;">32</span>
|-
|'''Offset'''<br>31&nbsp;&nbsp;&nbsp;<span style="float: right;">16</span>
|style="text-align: center; vertical-align: top;"|'''P'''
|'''DPL'''<br>1&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|style="text-align: center; vertical-align: top;"|0
|'''Gate Type'''<br>3&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|style="vertical-align:top"|Reserved
|'''IST'''<br>2&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|-
!style="text-align: left;" |31&nbsp;&nbsp;&nbsp;<span style="float: right;">16</span>
!style="text-align: left;" colspan="6" |15&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|-
|'''Segment Selector'''<br>15&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|colspan="6" |'''Offset'''<br>15&nbsp;&nbsp;&nbsp;<span style="float: right;">0</span>
|}

* '''Offset:''' A 64-bit value, split in three parts. It represents the address of the entry point of the '''[[Interrupt Service Routines|Interrupt Service Routine]]'''.
* '''Selector:''' A '''[[Segment Selector]]''' with multiple fields which must point to a valid code segment in your '''[[GDT]]'''.
* '''IST:''' A 3-bit value which is an offset into the '''Interrupt Stack Table''', which is stored in the '''[[Task State Segment]]'''. If the bits are all set to zero, the '''Interrupt Stack Table''' is not used.
* '''Gate Type:''' A 4-bit value which defines the type of gate this '''Interrupt Descriptor''' represents. In long mode there are two valid type values:
** '''0b1110''' or '''0xE''': 64-bit Interrupt Gate
** '''0b1111''' or '''0xF''': 64-bit Trap Gate
* '''DPL:''' A 2-bit value which defines the '''[[Security#Rings|CPU Privilege Levels]]''' which are allowed to access this interrupt via the '''INT''' instruction. Hardware interrupts ignore this mechanism.
* '''P:''' Present bit. Must be set ('''1''') for the descriptor to be valid.

In your '''[[Interrupt Service Routines|Interrupt Service Routines]]''', remember to return from the interrupt using the '''IRETQ''' instruction instead of '''IRET''', as assemblers will not translate that for you. Many 64-bit IDT related problems on the forum are caused by that missing 'Q'. Don't let this happen to you.

For more information, see '''Section 6.14.1: 64-Bit Mode IDT''' and '''Figure 6-8: 64-Bit IDT Gate Descriptors''' of the Intel Software Developer Manual, Volume 3-A.

=== Example Code ===

C Struct:

<syntaxhighlight lang="c">
struct InterruptDescriptor64 {
   uint16_t offset_1;        // offset bits 0..15
   uint16_t selector;        // a code segment selector in GDT or LDT
   uint8_t  ist;             // bits 0..2 holds Interrupt Stack Table offset, rest of bits zero.
   uint8_t  type_attributes; // gate type, dpl, and p fields
   uint16_t offset_2;        // offset bits 16..31
   uint32_t offset_3;        // offset bits 32..63
   uint32_t zero;            // reserved
};
</syntaxhighlight>

Example ''type_attributes'' values that people are likely to use (assuming DPL is 0):
* 64-bit '''Interrupt Gate: 0x8E''' (p=1, dpl=0b00, type=0b1110 => type_attributes=0b1000_1110='''0x8E''')
* 64-bit '''Trap Gate: 0x8F''' (p=1, dpl=0b00, type=0b1111 => type_attributes=1000_1111b='''0x8F''')

== IDT items ==

Originally created for Wikipedia. [https://en.wikipedia.org/w/index.php?title=Template:X86_protected_mode_interrupts&oldid=1283313564]

Copied from: [https://wiki.jwo.cz/wiki/x86_protected_mode_interrupts]

This table contains information collected from the ''Intel® 64 and IA-32 Architectures Software Developer’s Manual Volume 3 (3A, 3B, 3C & 3D): System Programming Guide'' revision from April&nbsp;2022.

{{x86 protected mode interrupts}}

== Gate Types ==

There are basically two kinds of interrupts: ones that occur when code execution has encountered an '''[[Exceptions|Exception]]''' due to bad code, or ones that occur to handle events unrelated to currently executing code. In the first case it is pertinent to save the address of the ''currently'' executing instruction so that it can be retried, these are called '''Traps'''. In the second case it is pertinent to save the address of the ''next'' instruction so that execution can be resumed where it left off. These could be caused by an IRQ or other hardware event, or by use of the '''INT''' instruction. Another difference to note is that with '''Traps''', new interrupts might occur during the service routine, but when the CPU is serving an IRQ, further interrupts are masked until an '''End of Interrupt''' signal is sent. How a certain interrupt is served depends on which kind of gate you put in the IDT entry.

=== Interrupt Gate ===

An '''Interrupt Gate''' is used to specify an '''[[Interrupt Service Routines|Interrupt Service Routine]]'''. For example, when the assembly instruction '''INT 50''' is performed while running in protected mode, the CPU looks up the 50th entry (located at 50 * 8) in the '''IDT'''. Then the Interrupt Gate's '''Selector''' and '''Offset''' values are loaded. The '''Selector''' and '''Offset''' are used to call the '''Interrupt Service Routine'''. When the '''IRET''' instruction is performed, the CPU returns from the interrupt. If the CPU was running in 32-bit mode and the specified selector is a 16-bit gate, then the CPU will go in 16-bit Protected Mode after calling the '''ISR'''. To return in this case, the '''O32 IRET''' instruction should be used, or else the CPU will not know that it should do a 32-bit return (reading 32-bit values off the [[stack]] instead of 16 bit).

=== Trap Gate ===

A '''Trap Gate''' should be used to handle '''[[Exceptions|Exceptions]]'''. When such an exception occurs, there can sometimes be an error code placed on the stack, which should be popped before returning from the interrupt.

'''Trap Gates''' and '''Interrupt Gates''' are similar, and their descriptors are structurally the same, differing only in the '''Gate Type''' field. The difference is that for '''Interrupt Gates''', interrupts are automatically disabled upon entry and reenabled upon '''IRET''', whereas this does not occur for '''Trap Gates'''.

=== Task Gate ===

A '''Task Gate''' is a gate type specific to IA-32 that is used for hardware task switching. For a '''Task Gate''' the '''Selector''' value should refer to a position in the '''[[GDT]]''' which specifies a '''[[Task State Segment]]''' rather than a code segment, and the '''Offset''' value is unused and should be set to zero. Rather than jumping to a service routine, when the CPU processes this interrupt, it will perform a hardware task switch to the specified task. A pointer back to the task which was interrupted will be stored in the '''Task Link''' field in the '''TSS'''.

{{Quotation|"*NOTE* Because IA-32 tasks are not re-entrant, an interrupt-handler task must disable interrupts between the time it completes handling the interrupt and the time it executes the IRET instruction. This action prevents another interrupt from occurring while the interrupt task's TSS is still marked busy, which would cause a general-protection (#GP) exception."|Intel Software Developer Manual}}

This type of gate is not often used as hardware task switching is slow and has little to no optimization on modern processors. As well, it has been entirely removed on [[X86-64|x86-64]].

==See Also==
===Articles===

* [[GDT]]
* [[IDT_problems|IDT problems]]

=== External References ===

* [http://www.logix.cz/michal/doc/i386/chp09-00.htm Michal Ludvig's Intel 80386 Programmer's Reference Manual chapter 9]

[[Category:X86 CPU]]
[[Category:Interrupts]]
[[de:Interrupt_Descriptor_Table]]
````

<a id="w001546"></a>

## W001546 — IDT

Bron-ID: W001546
Namespace: Main (0)
Revisie: 1961 | Datum: 2006-12-08T12:53:38Z
Bron: https://wiki.osdev.org/IDT
Revisie-URL: https://wiki.osdev.org/index.php?title=IDT&oldid=1961
Doorverwijzing: Interrupt Descriptor Table

````text
#REDIRECT [[Interrupt Descriptor Table]]
````

<a id="w001547"></a>

## W001547 — Interrupts Descriptor Table

Bron-ID: W001547
Namespace: Main (0)
Revisie: 1963 | Datum: 2006-12-08T12:54:27Z
Bron: https://wiki.osdev.org/Interrupts_Descriptor_Table
Revisie-URL: https://wiki.osdev.org/index.php?title=Interrupts_Descriptor_Table&oldid=1963
Doorverwijzing: Interrupt Descriptor Table

````text
#REDIRECT [[Interrupt Descriptor Table]]
````

<a id="w001548"></a>

## W001548 — Interrupt descriptor table

Bron-ID: W001548
Namespace: Main (0)
Revisie: 1965 | Datum: 2006-12-08T12:56:11Z
Bron: https://wiki.osdev.org/Interrupt_descriptor_table
Revisie-URL: https://wiki.osdev.org/index.php?title=Interrupt_descriptor_table&oldid=1965
Doorverwijzing: Interrupt Descriptor Table

````text
#REDIRECT [[Interrupt Descriptor Table]]
````

<a id="w001549"></a>

## W001549 — IVT

Bron-ID: W001549
Namespace: Main (0)
Revisie: 1967 | Datum: 2006-12-08T13:05:35Z
Bron: https://wiki.osdev.org/IVT
Revisie-URL: https://wiki.osdev.org/index.php?title=IVT&oldid=1967
Doorverwijzing: Interrupt Vector Table

````text
#REDIRECT [[Interrupt Vector Table]]
````

<a id="w001550"></a>

## W001550 — Interrupt Vector Table

Bron-ID: W001550
Namespace: Main (0)
Revisie: 30918 | Datum: 2026-08-22T04:25:58Z
Bron: https://wiki.osdev.org/Interrupt_Vector_Table
Revisie-URL: https://wiki.osdev.org/index.php?title=Interrupt_Vector_Table&oldid=30918

````text
On the [[x86]] architecture, the '''Interrupt Vector Table (IVT)''' is a table that specifies the addresses of all the 256 interrupt handlers used in [[real mode]].

The IVT is typically located at <tt>0000:0000H</tt>, and is 400H bytes in size (4 bytes for each interrupt). Although the default address can be changed using the LIDT instruction on newer CPUs, this is usually not done because it is both inconvenient and incompatible with other implementations and/or older software (e.g. MS-DOS programs). However, note that the code must remain in the first MiB of RAM.

== Structure ==

The entries are consecutive, meaning the first entry pointed by the IDTR is interrupt handler 0, and the others follow in succession. The format of an entry is:

  +-----------+-----------+
  |  Segment  |  Offset   |
  +-----------+-----------+
  4           2           0

We can therefore see that it's really easy to get the address of the interrupt handler we're looking for: IDTR * 4. In order to change an interrupt handler, all that needs to be done is to change its address in the table.

== CPU Interrupt Layout ==
{| class="wikitable"
|+ IVT Handlers
! Offset
! INT #
! Description
|-
| 0x0000
| 0x00
| Divide by 0
|-
| 0x0004
| 0x01
| Reserved
|-
| 0x0008
| 0x02
| NMI Interrupt
|-
| 0x000C
| 0x03
| Breakpoint (INT3)
|-
| 0x0010
| 0x04
| Overflow (INTO)
|-
| 0x0014
| 0x05
| Bounds range exceeded (BOUND)
|-
| 0x0018
| 0x06
| Invalid opcode (UD2)
|-
| 0x001C
| 0x07
| Device not available (WAIT/FWAIT)
|-
| 0x0020
| 0x08
| Double fault
|-
| 0x0024
| 0x09
| Coprocessor segment overrun
|-
| 0x0028
| 0x0A
| Invalid TSS
|-
| 0x002C
| 0x0B
| Segment not present
|-
| 0x0030
| 0x0C
| Stack-segment fault
|-
| 0x0034
| 0x0D
| General protection fault
|-
| 0x0038
| 0x0E
| Page fault
|-
| 0x003C
| 0x0F
| Reserved
|-
| 0x0040
| 0x10
| BIOS video services<ref group="note">If [[CR0]].NE is set to 1, FPU errors trigger interrupt 0x10 instead of 0x75. To avoid using the same interrupt for both FPU errors and BIOS video services, CR0.NE is typically 0 in real mode.</ref>
|-
| 0x0044
| 0x11
| Alignment check
|-
| 0x0048
| 0x12
| Machine check
|-
| 0x004C
| 0x13
| SIMD Floating-Point Exception
|-
| 0x00xx
| 0x14-0x1F
| Reserved
|-
| 0x0xxx
| 0x20-0xFF
| User definable
|}

==Default Hardware Interrupt Layout==

=== Master [[8259_PIC|8259]] ===

Some interrupts mapped by the 8259 by default overlap with some of the processor's exception handlers. These can be remapped via the 8259's IO ports.

{| class="wikitable"
|+
! IVT Offset
! INT #
! IRQ #
! Description
|-
| 0x0020
| 0x08
| 0
| PIT
|-
| 0x0024
| 0x09
| 1
| Keyboard
|-
| 0x0028
| 0x0A
| 2
| 8259A slave controller
|-
| 0x002C
| 0x0B
| 3
| COM2 / COM4
|-
| 0x0030
| 0x0C
| 4
| COM1 / COM3
|-
| 0x0034
| 0x0D
| 5
| LPT2
|-
| 0x0038
| 0x0E
| 6
| Floppy controller
|-
| 0x003C
| 0x0F
| 7
| LPT1
|-
|}

=== Slave 8259 ===

{| class="wikitable"
|+
! IVT Offset
! INT #
! IRQ #
! Description
|-
| 0x01c0
| 0x70
| 8
| RTC
|-
| 0x01C4
| 0x71
| 9
| Unassigned
|-
| 0x01C8
| 0x72
| 10
| Unassigned
|-
| 0x01CC
| 0x73
| 11
| Unassigned
|-
| 0x01D0
| 0x74
| 12
| Mouse controller
|-
| 0x01D4
| 0x75
| 13
| Math coprocessor
|-
| 0x01D8
| 0x76
| 14
| Hard disk controller 1
|-
| 0x01DC
| 0x77
| 15
| Hard disk controller 2
|-
|}

== See also ==

*[[Interrupt Descriptor Table]]
*[[Real mode]]
*[[8259 PIC]]

== Notes ==

<references group="note"/>

== External links ==

* [https://www.ctyme.com/intr/int.htm Interrupt Jump Table]

[[de:Interrupt Vector Table]]
[[Category:X86 CPU]]
````

<a id="w001551"></a>

## W001551 — Interrupt vector table

Bron-ID: W001551
Namespace: Main (0)
Revisie: 1969 | Datum: 2006-12-08T13:16:38Z
Bron: https://wiki.osdev.org/Interrupt_vector_table
Revisie-URL: https://wiki.osdev.org/index.php?title=Interrupt_vector_table&oldid=1969
Doorverwijzing: Interrupt Vector Table

````text
#REDIRECT [[Interrupt Vector Table]]
````

<a id="w001553"></a>

## W001553 — Text UI

Bron-ID: W001553
Namespace: Main (0)
Revisie: 30318 | Datum: 2026-03-17T19:03:37Z
Bron: https://wiki.osdev.org/Text_UI
Revisie-URL: https://wiki.osdev.org/index.php?title=Text_UI&oldid=30318

````text
A <b>Text User Interface</b> or <b>TUI</b> is a user interface where by all output is presented in the form of text, in contrast with a [[GUI|Graphical UI]] that uses graphics along with text to display output.

== Popularity and Use ==
Back in the DOS days before GUIs were widespread, almost all applications used some sort of text UI. Some of these were simple, showing a menu or unmovable panels, but others included a fully featured windowing system, like the [https://en.wikipedia.org/wiki/Turbo_Pascal Turbo Pascal IDE] or the MS-DOS edit application (you can't tell from the screenshot on the wiki, but file windows were actually resizable and movable and they were stacked).

With the rise of graphical UI's, text based user interfaces still remain practical in hobbyist operating system projects, as they are the most easy type of interface to implement, and are still in use in larger and commercial operating systems where a graphical user interface is unnecessary, for example, in large servers. A text user interface is also easy for quickly outputting results or debug data, and inputing commands to test new features, rather than writing complicated graphical applications.

These days text based UI is mostly used over pseudo-ttys, using [[Terminals|VT escape sequences]] rather than direct VGA buffers. Most notable examples are Midnight Commander (a Linux clone of the Norton Commander under DOS), and [https://vtm.netxs.online/ VTM] (a fully featured desktop environment). The biggest advantage of these UIs that they can be used over simple tty-based SSH tunnels, no additional protocol needed. Details on curses based text interfaces are out of the scope of this page, which only focuses on [[VGA Hardware|VGA]] buffer text interfaces.

== Input ==
Input in a text user interface primarily involves the use of a [[Command_Line|command line]] [[shell]], usually through entering commands via a [[keyboard]]. Other methods do exist, such as text-based menus that can scroll up and down, and prompts asking a user to press a specific key for a specific event to occur.

== Video Mode ==
The most used VGA video mode for a text UI is "VGA mode 3". This is the most commonly used, as it allows direct memory access to a linear address containing each character and its associated attributes. VGA mode 3 provides a text interface 80 characters wide and 25 characters lines per screen, although on modern computers [[Drawing In a Linear Framebuffer]] is preferrable, and often mandatory.

== Video Memory ==
In VGA mode 3, the linear text buffer is located in physical at 0xB8000. Reading and writing to and from this address will provide direct manipulation of on screen text. To access a particular character on the screen from X and Y coordinates is simple using the following formula:

<code>position = (y_position * characters_per_line) + x_position;</code>

Each character takes up two bytes of space in memory. The first byte is split into two segments, the forecolour, and the backcolour. The second byte is an 8-bit ASCII value of the character to print.

== Colours ==

Each character has a colour byte. This colour byte is split up in forecolour and backcolour.<br>

The layout of the byte, using the standard colour palette:
<pre>
Bit 76543210
    ||||||||
    |||||^^^-fore colour
    ||||^----fore colour bright bit
    |^^^-----back colour
    ^--------back colour bright bit OR enables blinking Text
</pre>

Its easy to write to BL, the Colour Nibbles(4Bit), in a Hex Value. <br>
For Example:<br>
 0x01 sets the background to black and the fore colour to blue<br>
 0x10 sets the background to blue and the fore colour to black <br>
 0x11 sets both to blue.
The default display colours set by the [[BIOS]] upon booting are 0x0F: 0 (black) for the background and 7 (White) + 8 (Bright) for the foreground.<br>
<br>

In text mode 0, the following standard colour palette is available for use. You can change this palette with VGA commands.
{| border="1" cellspacing="0"
! Number
! Colour
! Name
! Number + bright bit
! bright Colour
! Name
|- 
| 0
| style="background: #000000" | 
| Black
| 0+8=8
| style="background: #555555" | 
| Dark Gray
|-
| 1
| style="background: #0000AA" | 
| Blue
| 1+8=9
| style="background: #5555FF" | 
| Light Blue
|-
| 2
| style="background: #00AA00" | 
| Green
| 2+8=A
| style="background: #55FF55" | 
| Light Green
|-
| 3
| style="background: #00AAAA" | 
| Cyan
| 3+8=B
| style="background: #55FFFF" | 
| Light Cyan
|-
| 4
| style="background: #AA0000" | 
| Red
| 4+8=C
| style="background: #FF5555" | 
| Light Red
|-
| 5
| style="background: #AA00AA" | 
| Magenta
| 5+8=D
| style="background: #FF55FF" | 
| Light Magenta
|-
| 6
| style="background: #AA5500" | 
| Brown
| 6+8=E
| style="background: #FFFF00" | 
| Yellow
|-
| 7
| style="background: #AAAAAA" | 
| Light Gray
| 7+8=F
| style="background: #FFFFFF" | 
| White
|-
|}

== Example Console Class ==
A heavily documented and easy to understand example of a text console class is taken from Brandon's Kernel Tutorial, available at http://osdever.net/bkerndev/Docs/printing.htm

== C Code to Print a Character on the Screen ==
The following C code will print a character at X and Y coordinates:

<syntaxhighlight lang="c">
void WriteCharacter(unsigned char c, unsigned char forecolour, unsigned char backcolour, int x, int y)
{
     uint16_t attrib = (backcolour << 4) | (forecolour & 0x0F);
     volatile uint16_t * where;
     where = (volatile uint16_t *)0xB8000 + (y * 80 + x) ;
     *where = c | (attrib << 8);
}
</syntaxhighlight>

== Scrolling ==
Scrolling is achieved through copying the second line of characters onto the first, the third onto the second, etc. Then clearing the last line of text with null characters (usually a blank space with the fore/back colours of 7 and 0, respectively). In text mode 0, this is accomplished by copying characters 80-159 in to memory position 0-79, 160-239 into 80-159, etc.

== Virtual Terminals ==
Virtual terminals are much the same as physical screens, but instead of writing to the linear text buffer referring to the screen, text is written to a linear text buffer stored in memory. Each virtual terminal has it's own text buffer, and the terminal currently being displayed on screen has it's buffer copied into the video buffer to display on screen. Having more than one virtual text buffer has other advantages, such as only having parts of multiple buffers being output onto the screen at the same time.

== Buffering Data ==
When working with a text-based interface, writing text directly into the video memory may eventually begin to create issues, especially when handling keyboard input, processing input from menus or dealing with multiple text mode consoles. This problem can usually be overcome by writing characters and colour data into a separate buffer (unparsed), and periodically flushing the buffer to the screen. There are multiple ways to implement a text buffer; a simple solution might feature a flat block of memory in which text and colour bytes are stored before being copied to the screen, though more a more advanced approach may include a per-line buffer, planar-buffer or a system of tracking updated lines or portions of the screen.

==See Also==
* [[VGA Hardware]]
* [[Text Mode Cursor]]

===External Links===
* [https://vtm.netxs.online/ VTM] Text-based Desktop Environment that uses curses and VT codes

[[Category:Text UI]]
[[Category:Video]]
````

<a id="w001554"></a>

## W001554 — Microsoft Virtual PC

Bron-ID: W001554
Namespace: Main (0)
Revisie: 28348 | Datum: 2023-10-02T21:00:48Z
Bron: https://wiki.osdev.org/Microsoft_Virtual_PC
Revisie-URL: https://wiki.osdev.org/index.php?title=Microsoft_Virtual_PC&oldid=28348

````text
{{Emulators}}

Microsoft Virtual PC is Microsoft's release of a product line they acquired from Connectix in February 2003. It natively provides support for Windows operating systems, but, using the 'Other' option, it is possible to install and run pretty much any Operating System. (Tested with Linux and several hobbyist OSes.)

It has a very similar feel to the [[VMware]] PC emulator, and provides most of the same functions, and is free. A free downloadable version of VPC is available from Microsoft's website.

Summary: It is okay, but doesn't have an integrated debugger. Pretty fast though :)

=== Pros/cons: ===
* '''PRO:''' It provides PCI configuration method 1 support and [[SMBIOS]] ([[Bochs]] seemingly does not)
* '''PRO:''' Fixed size disk images are very easy to work with and can be shared by Virtual PC and Bochs :)
* '''PRO:''' Supports "dynamically expanding" disk images that start at a miniature size and expand when data is added.
* '''PRO:''' If you integrate a simple debugger and serial I/O into your OS, you can debug it over a named pipe. Just enter \\.\pipe\yourpipename into the configuration page for COM1 or COM2, and connect your debugger (on the host PC) to the same pipe.
* '''PRO:''' Works on 64-bit OS WinXP, WinVista, Win7 and with updated version of VMM.SYS on Win8/8.1 [http://www.win3x.org/win3board/viewtopic.php?t=19227 link]
* '''CON:''' 15- and 24-bit graphics modes aren't supported in the emulated S3 Trio (but they are in the real one), they screw up the display. It looks like some kind of 1980's computer with a defective CRT. Oddly enough, 32-bit color '''_is_''' supported. See [http://blogs.msdn.com/virtual_pc_guy/archive/2005/05/09/415814.aspx this post] on Virtual PC Guy's MSDN blog. ([https://learn.microsoft.com/en-US/archive/blogs/virtual_pc_guy/configuring-x-org-xfree86-under-virtual-pc-virtual-server updated link])
* '''CON:''' It only supports a minimal set of hardware - just enough for decent use. It doesn't support VBE 3.0 (the one with protected-mode access), BIOS32 for PCI (although that is easily implemented without the BIOS), or loopback mode for the serial UARTs.

=== Bug in Virtual PC ===
Virtual PC will throw an undefined opcode exception if a multibyte-nop (e.g. nopl (%eax,%eax,1)) is executed. Recent compilers will use them as an optimization and may cause your Kernel to not run on VPC.

== Windows Virtual PC ==
Microsoft Virtual PC for Windows 7 was renamed to Windows Virtual PC. It works on 64 bit versions of Windows unlike it's predecessor, but still doesn't emulate a 64 bit environment. The main differences is lack of GUI floppy addition (if you want a floppy disk, you need to directly edit config). However, it has the advantage of a simple interface, and virtual PCs are treated as a folder under Windows 7, where you can  create a VM in the top bar of explorer.

[[Category:Emulators]]
````

<a id="w001555"></a>

## W001555 — VMware

Bron-ID: W001555
Namespace: Main (0)
Revisie: 29214 | Datum: 2024-08-11T22:01:38Z
Bron: https://wiki.osdev.org/VMware
Revisie-URL: https://wiki.osdev.org/index.php?title=VMware&oldid=29214

````text
{{Emulators}}

'''VMware''' is basically the rich man's ''virtualizer''. It can run any number of "Guest Operating Systems" and can boot from real media or image files. It comes with a GUI that makes it easier to configure than [[Bochs]] or [[QEMU]].

Please note: VMware is not an emulator. There is lot of difference between an emulator and a virtual machine monitor. Emulators, like Bochs, emulate each instruction, whereas VMware (and Virtual PC) try to run most of the code on the host PC directly, and only emulate instructions that create invalid instruction faults or access system-level memory.

The VMware BIOS supports booting from a CD (use mkisofs to make a bootable CD, attach cdrom device to the .iso file) or a floppy drive (attach the floppy disk to a file), plus hard drives.  These are the easiest options for loading your own kernel.

Both VMware Workstation Pro and VMware Fusion Pro are free for personal use as of May 14, 2024.[https://blogs.vmware.com/cloud-foundation/2024/05/14/vmware-desktop-hypervisor-pro-apps-now-available-for-personal-use/]

== Versions ==

VMware's usefulness for hobbyists depends on the code's generation.  Broadly:
* Workstation 5.0 (and previous).  Not at all helpful for hobbyists.  Error codes are designed for reporting to VMware, and there are no developer-centric features.
* Workstation 5.5 (Player 1.0, Server 1.0).  Marginally more useful: a buggy guest operating system traces useful error messages.  But in reality, go for something more modern.  Supports SMP, if you are writing a multiprocessor kernel.  Server and Player are free, which is a perk; go for Server.
* Workstation 6.0 (Fusion 1.0/1.1).  Useful feature: gdb-based guest debug stub support.  Useful feature: record-replay (though difficult to use), which might be helpful for reproducing race conditions.
* ??? (Server 2.0 beta in progress).  Current generation; details unavailable.
== Guest debugging ==
These options are valid in Workstation 6.0+, and should be set in the virtual machine's .vmx file.

* debugStub.listen.guest32 = "TRUE"
* debugStub.listen.guest64 = "TRUE"

If using these options, Workstation prints a message "VMware Workstation is listening for debug connection on port 8832." into the vmware.log file.  Start a GDB session (using a copy of your kernel that includes debug information), then:
* target remote localhost:8832

Note that some versions of VMWare contain a bug: after you disconnect GDB from port 8832, VMWare will actually continue listening on port 8833 not 8832. The actual port can always be located by looking at the latest "Debug stub" entry in the vmware.log file:

<code>vmx| W110: Debug stub: VMware Workstation is listening for debug connection on port 8833.</code>

Standard gdb commands work in this mode (e.g. printing memory, backtrace).  Note that this is a debug stub attached to the KERNEL, not a userspace program.

Other useful options:
* debugStub.listen.guest32.remote = "TRUE"   # Allows debugging from a different computer / VM instead of localhost. # The IP for remote debugging will be that of the host.
* debugStub.listen.guest64.remote = "TRUE"   # Same, but for 64-bit code
* monitor.debugOnStartGuest32 = "TRUE"       # Breaks into debug stub on first instruction (warning: in BIOS!) # This will halt the VM at the very first instruction at 0xFFFF0, you could set the next breakpoint to break *0x7c00 to break when the bootloader is loaded by the BIOS
* debugStub.hideBreakpoints = "TRUE"         # Enables the use of hardware breakpoints instead of software (INT3) breakpoints
* bios.bootDelay = "3000"                    # Delay booting the BIOS code.

== Triple faults ==

VMware products emulate a triple fault by resetting the machine.  On beta products, however, a guest triple fault results in Bug 19580.  Sometimes this represents a bug in VMware's emulation; usually, this represents a bug in your kernel.

== Guest Tools ==
{{Main|VMware_tools}}
As a virtualization product aimed at regular users, VMware provides a number of facilities through its "tools" package for major OSes, such as absolute mouse positioning and display resizing.

== Compatibility with IDEs ==
* IDA Pro supports debugging Windows kernels via the VMWare stub [https://www.hex-rays.com/products/ida/support/tutorials/debugging_gdb_linux_vmware.pdf]
* VisualKernel supports debugging Linux kernels from Visual Studio with VMWare gdb stub [http://visualkernel.com/tutorials/vmware/]

== See Also ==

=== External Links ===
* http://www.vmware.com/ - VMware homepage.
* http://communities.vmware.com/index.jspa - Community forums.  Engineers are present; if asked politely, some will provide very exact technical details
* http://www.pagetable.com/?p=25 - A description of a problem encountered using VMware on AMD Opteron RevC processors.
* http://stackframe.blogspot.com/2007/04/debugging-linux-kernels-with.html - VMware engineer describes how to debug Linux kernel with debug stub.
* http://sourceforge.net/projects/open-vm-tools/ - Open source client tools.  Good source for VMware proprietary hardware and host/client hooks
[[Category:Emulators]]
````

<a id="w001556"></a>

## W001556 — Bochs

Bron-ID: W001556
Namespace: Main (0)
Revisie: 30658 | Datum: 2026-05-28T15:31:09Z
Bron: https://wiki.osdev.org/Bochs
Revisie-URL: https://wiki.osdev.org/index.php?title=Bochs&oldid=30658

````text
{{Emulators}}

'''Bochs''' is a steadily improving emulator for the [[:Category:x86|x86]] platform. It greatly improves OS development because the virtual machine restarts much quicker than real hardware, plus it offers detailed debugging functionality that can help greatly during kernel development.

It is strongly suggested to get two installations of Bochs - in addition to the out-of-the-box version (which might well be one of the binary downloads), you should also compile a second instance with the internal debugger enabled - so when something strange occurs that your 'normal' debugging tools can't trace, you can cross-check it with the debug version. You may also wish to enable (and use) the debug IO ports.


==Frequent error messages==
===Running in Bogus Memory===
You sent your code pointer (EIP) to some uninitialized memory area. This means you either followed a NULL (or uninitialized) pointer, or you damaged the return address of your [[Stack#Stack example on the X86 architecture|stack frame]]. Make your code more clean, test pointers before you follow them, initialize every pointer (especially those who are on the stack) and enable *-Wall* in GCC.

=== 3rd exception with no resolution===

The CPU didn't manage to invoke an exception handler and would normally [[Triple Fault|triple fault]]. This is probably due to a bad [[Interrupt Descriptor Table|IDT]] register content, or a bad IDT descriptor. Sometimes (but less likely), it can also be due to a severe bug in your exception handler code. Check your exception works with "illegal" asm instructions like <tt>idiv 0</tt>, or

<syntaxhighlight lang="nasm">
push 0xf001
pop ds                   ; 0xf001 is no valid segment,
mov ax, ~[ds:0x12345678] ; let's see if we get the GPF
</syntaxhighlight>

In several cases, there are other error messages prior to this one which can provide more details in the error. Some common messages that might be displayed:

* interrupt(): gate descriptor is not valid sys seg
: You have not loaded an IDT, or the IDT is corrupt
* interrupt(): SS selector null
:* You have no TSS
:* You haven't set SS0 / ESP0 in the TSS
* CR0 = 0xe0000001 CR2 = 0xe0000001
:* Your page tables are not page-aligned 
:* Your page tables do not point to the correct parts of memory

===I/O Operand Size===
Bochs performs some rather paranoid checks on I/O operand size. Reading a byte from port 0x1234 is usually not the same thing as reading a 32-bit value. Go back to your chip's data sheet and double-check that your sizes are correct.

=== fetch_raw_descriptor: LDTR.valid=0 ===
Many of you have said "but ... I do not have an LDT and I read it wasn't mandatory!?". You're right. And so is Bochs. This message usually means that your program tried to load a selector with some garbage value, which happened to have the 3rd bit (Table Indicator) set. The CPU will try to look up the descriptor in the LDT, but there's no LDT registered! In most cases, the error comes from some mispairing of push and pop on the stack, which lead to a non-selector value to be loaded in a segment register.

If you're still stuck, download the Bochs source package and search for the message you received. Then, maybe you can add extra information to the message-printing code (like the faulty offset of a segfault, the segment limit, etc). But keep away from modifying Bochs' operations! Every time I suspected a bug in Bochs, I was just misunderstanding the Intel Manual...

==Differences between Bochs and real hardware==
;Bochs enables the A20 line in the BIOS
:Your PC doesn't necessarily do so. Sometimes there's a BIOS option, sometimes there isn't. Check your code that enables the A20 line and make sure it has no issues with faster hardware.

;Bochs wipes out its memory
:In Bochs, memory is always filled with zero until you (or the BIOS) put something else. On a normal PC, uninitialized memory tends to contain garbage (most often all-ones) or traces of previous executions. Check your pointers, initialize them, print your code on paper if necessary.

;Bochs does not properly emulate CPU cache/TLB
:Although Bochs does have these constructs, they do not work the same as a regular CPU cache or TLB and do not change based on which CPU is being emulated.  If you're not handling caching or TLB refreshes correctly, behavior may differ in Bochs vs. hardware (i.e. it may work on Bochs but not on hardware, or vice versa).

;Bochs floppy has no errors
:In a physical PC, it's common to issue up to 3 read commands on a sector/track before it can be read fine. If you don't have proper error check/recovery in your bootsector, you're likely to run something that is not your kernel...

;Bochs is flexible about returning to real mode
:Despite what Chris Giese's Protected Mode tutorial states, you do not have to be in 16-bit protected mode to clear the PE bit of CR0. If you fail to enter 16-bit protected mode on a real PC, it will hang, without giving any error indication - no triple fault or anything!

;Bochs' timer is not real time
:(unless you configure it closer to real time). Waiting for 2 second on bochs will let any virtual device that needs 2 seconds to be ready be ready, but that could be just 0.02 seconds for you ... or that could be 200 seconds.

Adding 
 clock: sync=realtime, time0=local
will cause interrupts at real intervals, but may overload the emulated OS with PIT interrupts and overflow the stack (if it's pre-emptible) Using <tt>sync=slowdown</tt> will also set it to real-time speeds, but with a constant amount of virtual clock cycles between each virtual second. For both the slowdown and original clock, you need to set the amount of instructions executed per second to the desired value.

;CPU is always Intel/AMD
:Bochs emulates a CPU, so CPUID will always report back Intel (if emulating a 32-bit system) or AMD (if emulating a 64-bit system) regardless of what is really in your system.
Newer versions of Bochs allow you to specify the vendor string in the CPU tag:
 cpu: vendor_string="test        "
 # must be a twelve-character ASCII string!

==Bochs image files in a nutshell==

To use Bochs to boot your custom kernel, search for the "floppya" and "boot" lines in the bochsrc configuration file. The following configuration allows you to use, and boot from, the "real" floppy drive of your computer:

 floppya: 1_44=/dev/fd0, status=inserted
 boot: floppy

For Windows users, that should read:

 floppya: 1_44=a:, status=inserted
 boot: floppy

But this is awfully slow, and puts lots of stress on the floppy when you are in a tight patch - make - boot cycle. Bochs offers the use of image files, including an interactive tool to create an image file (bximage.exe). Note that Bochs will emulate the floppy's internals even when the image is a device rather than a regular file ...

==Bochs debugging facilities==
Bochs has several features that eases debugging. Many of them must be enabled via an configure switch:

===GUI debugger===
Bochs has a command-line internal debugger, and a graphical interface on top of it.  You must compile Bochs with the configuration option and then edit your bochsrc file in order to enable the GUI debugger.  I use this line in my bochsrc file to enable graphical debugging in X:

  display_library: x, options="gui_debug"

In a Windows environment, add this line to your bochsrc.bxrc

  display_library: win32, options="gui_debug"

It seems that on Windows, the "option" flag (what the above line used to read) will be accepted, but the GUI window will not appear.

===I/O debugger macros===
Some useful macros when Bochs is compiled with the I/O debug ports enabled (<tt>port_e9_hack: enabled=1</tt> if Bochs 2.4 or newer, <tt>configure --port-e9-hack</tt> if not):

<syntaxhighlight lang="c">
 //outputs a character to the debug console
 #define BochsConsolePrintChar(c) outportb(0xe9, c)
 //stops simulation and breaks into the debug console
 #define BochsBreak() outportw(0x8A00,0x8A00); outportw(0x8A00,0x08AE0);
</syntaxhighlight>

===Magic Breakpoint===
When you're using Bochs with the [http://bochs.sourceforge.net/doc/docbook/user/internal-debugger.html internal debugger], you can trigger the debugger via a facility called [http://bochs.sourceforge.net/doc/docbook/user/bochsrc.html#AEN2324 magic breakpoints]. To trigger a breakpoint, you can insert <tt>xchg bx, bx</tt> (in GAS syntax, <tt>xchgw %bx, %bx</tt>) anywhere in the code and Bochs will trap into the debugger as soon as it executes it. On real hardware this has no effect as it merely replaces the BX register with itself.

You should put the following line in your Bochs configuration file to have it listen to magic breakpoints:
 magic_break: enabled=1

On older versions, enabling the debugger alone doesn't compile in magic breakpoint support, you will also need to specify <tt>--enable-magic-breakpoint</tt> when configuring the build on those versions.

===Debugging SMP===
When using the internal debugger, you may use the following command to switch CPUs:

  set $cpu = <n>

I also found this was necessary on occasion in the GUI debugger, even though it has buttons for each CPU.

===Internal debugger commands===

You can pass a file containing debug commands to automatically run whenever you start bochs with the internal debugger.  (bochs -rc <file>)

Bochs places an automatic breakpoint just before the BIOS loads, this can be automatically skipped by putting <tt>continue</tt> as the first command in the said file.

===Debugging Triple Faults===

When using the internal debugger, you may change this line in your Bochs configuration file:

 reset_on_triple_fault 0

This line disables the emulator reset on a Triple fault, enabling you to debug the code after a Triple fault occured (Very useful while implementing paging).
==Compiling Bochs from Source==
Bochs has many compile-time configuration options, some of which conflict, and therefore a binary distribution of Bochs may not be suitable for your purposes.  I found it was best to compile my own copy of Bochs to be sure I had the features that I needed.  Also, you should consider using the CVS snapshot version of Bochs if the released version is old and not working for you.  I found this was necessary up until version 2.4 was released, for example.  On Ubuntu, you may have to run

<syntaxhighlight lang="shell-session">
$ sudo apt-get install libgtk2.0-dev
</syntaxhighlight>

and enter your password.  On other linux distros, try the equivalent.

The array of Bochs configuration options can be confusing, and you cannot assume the defaults are going to be sensible.  These are the options I use, this can get you started:

<syntaxhighlight lang="bash">
bochs  ./configure --enable-smp \
              --enable-cpu-level=6 \
              --enable-all-optimizations \
              --enable-x86-64 \
              --enable-pci \
              --enable-vmx \
              --enable-debugger \
              --enable-debugger-gui \
              --enable-logging \
              --enable-fpu \
              --enable-3dnow \
              --enable-sb16=dummy \
              --enable-cdrom \
              --enable-x86-debugger \
              --enable-iodebug \
              --disable-plugins \
              --disable-docbook \
              --enable-sb16 \
              --enable-e1000 \
              --enable-usb \
              --with-x --with-x11 --with-term --with-sdl2
</syntaxhighlight>

A few notes:
* If you are on Windows, that last line should probably read "--with-win32".
* On Linux, using SDL as the display library over X11 is preferable as the performance appears to increase greatly on some setups
* Bochs has GDB stub support, and its own internal debugger.  These cannot be compiled into the same Bochs binary.  The internal debugger is very useful, its flag is --enable-debugger
* The GDB stub in Bochs does not support SMP, last time I checked.
* If you do not enable PCI, then the Intel Multiprocessing tables will not appear in memory.
* I was unable to successfully load the GUI debugger without specifying <tt>--disable-plugins</tt>.  Otherwise, I get dynamic loading symbol errors.
* Post-2.4.2 several of the CPU specific options were folded into the CPU-level specification and are therefore deprecated.  They have been removed from the example above.
* The default compile does not support x86-64, --enable-x86-64 will turn it on
* On many Linux distributions it is possible to install Bochs via a package manager. For example, on distributions that use apt-get we can do
<syntaxhighlight lang="shell-session">
$ sudo apt-get install bochs
$ sudo apt-get install bochs-x
</syntaxhighlight>
to install Bochs and the X11 plugin (which may crash on ubuntu/linux mint: install the sdl plugin and use sdl instead of x as the display library in this case). Note that there is a big chance that the graphical debugger is not enabled in the binaries from the package manager.

==See Also==
===Articles===
*[[Bochs Graphics Adaptor]]
*[[Emulator Comparison]]
*[[GDB]]
*[[:Category:Disk Image Utilities|Category:Disk Image Utilities]]

===External Links===
*[http://bochs.sourceforge.net Bochs Homepage]
*[http://bochs.sourceforge.net/doc/docbook/user/internal-debugger.html Bochs internal debugger commands documentation]
*[http://www.codeproject.com/system/MakingOS_2.asp CodeProject article on using Bochs]

[[Category:Emulators]]
[[de:Bochs]]
````

<a id="w001557"></a>

## W001557 — Template:Show1

Bron-ID: W001557
Namespace: Template (10)
Revisie: 28994 | Datum: 2024-06-06T17:27:30Z
Bron: https://wiki.osdev.org/Template:Show1
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AShow1&oldid=28994

````text
{{{1}}}
````

<a id="w001558"></a>

## W001558 — Template:If

Bron-ID: W001558
Namespace: Template (10)
Revisie: 29029 | Datum: 2024-06-06T20:16:17Z
Bron: https://wiki.osdev.org/Template:If
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AIf&oldid=29029

````text
{{show1
|1={{{2|{{{then|}}}}}}
|1{{{1|{{{test|}}}}}}={{{3|{{{else|}}}}}}
}}
````

<a id="w001559"></a>

## W001559 — Template:Infobox Tool

Bron-ID: W001559
Namespace: Template (10)
Revisie: 15042 | Datum: 2013-08-10T05:44:48Z
Bron: https://wiki.osdev.org/Template:Infobox_Tool
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AInfobox_Tool&oldid=15042

````text
<table align="right" cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; width: 30em; font-size: 90%; text-align: left;">
<tr><td colspan="2" style="text-align: center; font-size: 130%; font-weight: bold;">
{{{name}}}
</td></tr>
{{if | {{{image}}} | <tr><td colspan="2">[[Image:{{{image}}}|300px|]]</td></tr>}}
{{if | {{{operating_system|}}} | <tr><td>'''OS:'''</td><td>{{{operating_system}}}</td></tr>}}
{{if | {{{website|}}} | <tr><td>'''Website:'''</td><td>{{{website}}}</td></tr>}}
</table>
````

<a id="w001560"></a>

## W001560 — Programmable Interval Timer

Bron-ID: W001560
Namespace: Main (0)
Revisie: 30843 | Datum: 2026-07-03T00:12:58Z
Bron: https://wiki.osdev.org/Programmable_Interval_Timer
Revisie-URL: https://wiki.osdev.org/index.php?title=Programmable_Interval_Timer&oldid=30843

````text
{{Historical}}

The '''Programmable Interval Timer''' ('''PIT''') chip (Intel 8253/8254) basically consists of an oscillator, a prescaler and 3 independent frequency dividers. Each frequency divider has an output, which is used to allow the timer to control external circuitry (for example, IRQ 0).

On modern systems, the '''PIT''' has been obsoleted by both the [[HPET]] timer which offers higher precision and performance, the [[ACPI Timer]] which is easier to detect, and by the [[APIC Timer]] which is multiprocessor aware. In addition, PIT has been found to be broken (at least on Arrow Lake) or absent (at least on Surface Pro 4 <ref>[https://www.kernel.org/pub/linux/kernel/v4.x/ChangeLog-4.9.104]</ref>) on modern hardware, so assuming its presence is no good.

== The Oscillator ==
The oscillator used by the PIT chip runs at (roughly) 1.193182 MHz. The reason for this requires a trip back into history (to the later half of the 1970's)...

The original PC used a single "base oscillator" to generate a frequency of 14.31818 MHz because this frequency was commonly used in television circuitry at the time. This base frequency was divided by 3 to give a frequency of 4.77272666 MHz that was used by the CPU, and divided by 4 to give a frequency of 3.579545 MHz that was used by the CGA video controller. By logically ANDing these signals together a frequency equivalent to the base frequency divided by 12 was created. This frequency is 1.1931816666 MHz (where the 6666 part is recurring). At the time it was a brilliant method of reducing costs, as the 14.31818 MHz oscillator was cheap due to mass production and it was cheaper to derive the other frequencies from this than to have several oscillators. In modern computers, where the cost of electronics is much less, and the CPU and video run at much higher frequencies the PIT lives on as a reminder of "the good ole' days".

== Frequency Dividers ==
The basic principle of a frequency divider is to divide one frequency to obtain a slower frequency. This is typically done by using a counter. Each "pulse" from the input frequency causes the counter to be decreased, and when that counter has reached zero a pulse is generated on the output and the counter is reset. For example, if the input signal is 200 Hz and the counter is reset to a value of ten each time, then the output frequency would be 200/10, or 20 Hz.

The PIT has only 16 bits that are used as frequency divider, which can represent the values from 0 to 65535. Since the frequency can't be divided by 0 in a sane way, many implementations use 0 to represent the value 65536 (or 10000 when programmed in BCD mode).

The PIT chip has three separate frequency dividers (or 3 separate channels) that are programmable, in that the value of the "reset counter" is set by software (the OS). Software also specifies an action to be taken when the counter reaches zero on each individual channel. In this way, each channel can be used in one of several "modes" – for example, as a frequency divider (where the count is automatically reset) or as a "one shot" timer (where the count isn't automatically reset).

Each PIT channel also has a "gate input" pin which can be used to control whether the input signal (the 1.19MHz one) gets to the channel or not. For PIT channels 0 and 1, the associated gate input pin is not connected to anything. The PIT channel 2 gate is controlled by IO port 0x61, bit 0.

== PIT Timer Accuracy ==
The accuracy of the PIT timer depends on the quality of the oscillator used, and is typically accurate to within +/- 1.73 seconds per day. There are many causes for this inaccuracy, however because of this there isn't much point in specifying times or frequencies to more than five or six digits.

== Outputs ==
Channel 0 is connected directly to IRQ0, so it is best to use it only for purposes that should generate interrupts. Channel 1 is unusable, and may not even exist. Channel 2 is connected to the PC speaker, but can be used for other purposes without producing audible speaker tones.

=== Channel 0 ===
The output from PIT channel 0 is connected to the PIC chip, so that it generates an "IRQ 0". Typically during boot the BIOS sets channel 0 with a count of 65535 or 0 (which translates to 65536), which gives an output frequency of 18.2065 Hz (or an IRQ every 54.9254 ms). Channel 0 is probably the most useful PIT channel, as it is the only channel that is connected to an IRQ. It can be used to generate an infinte series of "timer ticks" at a frequency of your choice (as long as it is higher than 18 Hz), or to generate single CPU interrupts (in "one shot" mode) after programmable short delays (less than an 18th of a second).

When choosing [[#Operating Modes|an operating mode]], below, it is useful to remember that the IRQ0 is generated by the ''rising edge'' of the Channel 0 output voltage (ie. the transition from "low" to "high", only).

=== Channel 1 ===
The output for PIT channel 1 was once used (in conjunction with the DMA controller's channel 0) for refreshing the DRAM (Dynamic Random Access Memory) or RAM. Typically, each bit in RAM consists of a capacitor which holds a tiny charge representing the state of that bit, however (due to leakage) these capacitors need to be "refreshed" periodically so that they don't forget their state.

On later machines, the DRAM refresh is done with dedicated hardware and the PIT (and DMA controller) is no longer used. On modern computers where the functionality of the PIT is implemented in a large scale integrated circuit, PIT channel 1 is no longer usable and may not be implemented at all.

=== Channel 2 ===
The output of PIT channel 2 is connected to the PC speaker, so the frequency of the output determines the frequency of the sound produced by the speaker. This is the only channel where the gate input can be controlled by software (via bit 0 of I/O port 0x61), and the only channel where its output (a high or low voltage) can be read by software (via bit 5 of I/O port 0x61). Details of how to program the PC speaker can be found [[PC_Speaker|here]].

== I/O Ports ==
The PIT chip uses the following I/O ports:

 I/O port     Usage
 0x40         Channel 0 data port (read/write)
 0x41         Channel 1 data port (read/write)
 0x42         Channel 2 data port (read/write)
 0x43         Mode/Command register (write only, a read is ignored)

Each 8 bit data port is the same, and is used to set the counter's 16 bit reload value or read the channel's 16 bit current count (more on this later). The PIT channel's current count and reload value should not be confused. In general, when the current count reaches zero the PIT channel's output is changed and the current count is reloaded with the reload value, however this isn't always the case. How the current count and reload value are used and what they contain depends on which mode the PIT channel is configured to use.

The Mode/Command register at I/O address 0x43 contains the following:

 Bits         Usage
 7 and 6      Select channel :
                 0 0 = Channel 0
                 0 1 = Channel 1
                 1 0 = Channel 2
                 1 1 = Read-back command (8254 only)
 5 and 4      Access mode :
                 0 0 = Latch count value command
                 0 1 = Access mode: lobyte only
                 1 0 = Access mode: hibyte only
                 1 1 = Access mode: lobyte/hibyte
 3 to 1       Operating mode :
                 0 0 0 = Mode 0 (interrupt on terminal count)
                 0 0 1 = Mode 1 (hardware re-triggerable one-shot)
                 0 1 0 = Mode 2 (rate generator)
                 0 1 1 = Mode 3 (square wave generator)
                 1 0 0 = Mode 4 (software triggered strobe)
                 1 0 1 = Mode 5 (hardware triggered strobe)
                 1 1 0 = Mode 2 (rate generator, same as 010b)
                 1 1 1 = Mode 3 (square wave generator, same as 011b)
 0            BCD/Binary mode: 0 = 16-bit binary, 1 = four-digit BCD

The "Select Channel" bits select which channel is being configured, and must always be valid on every write of the mode/command register, regardless of the other bits or the type of operation being performed. The "read back" (both bits set) is not supported on the old 8253 chips but should be supported on all AT and later computers except for PS/2 (i.e. anything that isn't obsolete will support it). The "read back" command is discussed later.

The "Access Mode" bits tell the PIT what access mode you wish to use for the selected channel, and also specify [[#Counter Latch Command|the "counter latch" command]] to the CTC. These bits must be valid on every write to the mode/command register. For [[#Read Back Command|the "read back" command]], these bits have a different meaning. For the remaining combinations, these bits specify what order data will be read and written to the data port for the associated PIT channel. Because the data port is an 8 bit I/O port and the values involved are all 16 bit, the PIT chip needs to know what byte each read or write to the data port wants. For "lobyte only" and "hibyte only", only the lowest/highest 8 bits of the counter value is read or written to/from the data port – the respective other 8 bits are zeroed (e.g. with "lobyte only", the highest 8 bits will be zeroed). For the "lobyte/hibyte" mode, 16 bits are always transferred as a pair, with the lowest 8 bits followed by the highest 8 bits (both 8 bit transfers are to the '''same''' IO port, sequentially – a word transfer will not work).

The "Operating Mode" bits specify which mode the selected PIT channel should operate in. For the "read back" command and the "counter latch" command, these bits have different meanings (see the information corresponding to these commands below). There are 6 different [[#Operating Modes|operating modes]]. Each operating mode will be discussed separately later.

The "BCD/Binary" bit determines if the PIT channel will operate in binary mode or BCD mode (where each 4 bits of the counter represent a decimal digit, and the counter holds values from 0000 to 9999). 80x86 PCs only use binary mode (BCD mode is ugly and limits the range of counts/frequencies possible). Although it should still be possible to use BCD mode, it may not work properly on some "compatible" chips. For the "read back" command and the "counter latch" command, this bit has different meanings (see the information corresponding to these commands below).

== Operating Modes ==
While each operating mode behaves differently, some things are common to all operating modes. This includes:

; '''Initial Output State'''
: Every time the mode/command register is written to, all internal logic in the selected PIT channel is reset, and the output immediately goes to its initial state (which depends on the mode).
; '''Changing Reload Value'''
: A new reload value can be written to a PIT channel's data port at any time. The operating mode determines the exact effect that this will have.
; '''Current Counter'''
: The current counter value is always either decremented or reset to the reload value on the ''falling'' edge of the (1.193182 MHz) input signal.
; '''Current Counter Reload'''
: In modes where the current count is decremented when it is reloaded, the current count is not decremented on the same input clock pulse as the reload – it starts decrementing on the ''next'' input clock pulse.

=== Mode 0 – Interrupt On Terminal Count ===
For this mode, when the mode/command register is written the output signal goes low and the PIT waits for the reload register to be set by software, to begin the countdown. After the reload register has been set, the current count will be set to the reload value on the next falling edge of the (1.193182 MHz) input signal. Subsequent falling edges of the input signal will decrement the current count (if the gate input is high on the preceding rising edge of the input signal).

When the current count decrements from one to zero, the output goes high and remains high until another mode/command register is written or the reload register is set again. The current count will wrap around to 0xFFFF (or 0x9999 in BCD mode) and continue to decrement until the mode/command register or the reload register are set, however this will not affect the output pin state.

The reload value can be changed at any time. In "lobyte/hibyte" access mode counting will stop when the first byte of the reload value is set. Once the full reload value is set (in any access mode), the next falling edge of the (1.193182 MHz) input signal will cause the new reload value to be copied into the current count, and the countdown will continue from the new value.

Note: despite the misleading name of this mode, it only generates interrupts on channel 0.

=== Mode 1 – Hardware Re-triggerable One-shot ===
This mode is similar to mode 0 above, however counting doesn't start until a rising edge of the gate input is detected. For this reason it is not usable for PIT channels 0 or 1 (where the gate input can't be changed).

When the mode/command register is written the output signal goes high and the PIT waits for the reload register to be set by software. After the reload register has been set the PIT will wait for the next rising edge of the gate input. Once this occurs, the output signal will go low and the current count will be set to the reload value on the next falling edge of the (1.193182 MHz) input signal. Subsequent falling edges of the input signal will decrement the current count.

When the current count decrements from one to zero, the output goes high and remains high until another mode/command register is written or the reload register is set again. The current count will wrap around to 0xFFFF (or 0x9999 in BCD mode) and continue to decrement until the mode/command register or the reload register are set, however this will not affect the output pin state.

If the gate input signal goes low during this process it will have no effect. However, if the gate input goes high again it will cause the current count to be reloaded from the reload register on the next falling edge of the input signal, and restart the count again (the same as when counting first started).

The reload value can be changed at any time, however the new value will not affect the current count until the current count is reloaded (on the next rising edge of the gate input). So if you want to do this, clear and then reset bit 0 of IO port 0x61, after modifying the reload value.

=== Mode 2 – Rate Generator ===
This mode operates as a frequency divider.

When the mode/command register is written the output signal goes high and the PIT waits for the reload register to be set by software. After the reload register has been set, the current count will be set to the reload value on the next falling edge of the (1.193182 MHz) input signal. Subsequent falling edges of the input signal will decrement the current count (if the gate input is high on the preceding rising edge of the input signal).

When the current count decrements from two to one, the output goes low, and on the next falling edge of the (1.193182 MHz) input signal it will go high again and the current count will be set to the reload value and counting will continue.

If the gate input goes low, counting stops and the output goes high immediately. Once the gate input has returned high, the next falling edge on input signal will cause the current count to be set to the reload value and operation will continue.

The reload value can be changed at any time, however the new value will not affect the current count until the current count is reloaded (when it is decreased from two to one, or the gate input going low then high). When this occurs counting will continue using the new reload value.

A reload value (or divisor) of one must ''not'' be used with this mode.

This mode creates a high output signal that drops low for one input signal cycle (0.8381 uS), which is too fast to make a difference to the PC speaker (see mode 3). For this reason mode 2 is useless for producing sounds with PIT channel 2.

Typically, OSes and BIOSes use mode 3 (see below) for PIT channel 0 to generate IRQ 0 timer ticks, but some use mode 2 instead, to gain frequency accuracy (frequency = 1193182 / reload_value Hz).

=== Mode 3 – Square Wave Generator ===
For mode 3, the PIT channel operates as a frequency divider like mode 2, however the output signal is fed into an internal "flip flop" to produce a square wave (rather than a short pulse). The flip flop changes its output state each time its input state (or the output of the PIT channel's frequency divider) changes. This causes the actual output to change state half as often, so to compensate for this the current count is decremented twice on each falling edge of the input signal (instead of once), and the current count is set to the reload value twice as often.

When the mode/command register is written the output signal goes high and the PIT waits for the reload register to be set by software. After the reload register has been set, the current count will be set to the reload value on the next falling edge of the (1.193182 MHz) input signal. Subsequent falling edges of the input signal will decrement the current count twice (if the gate input is high on the preceding rising edge of the input signal).

Note: under normal circumstances the output state will be low 50% of the time when the mode/command register is written. The output will then go high, which will generate an immediate (perhaps spurious) IRQ0. The other 50% of the time the output will already be high, and there will be no IRQ0 generated.

For even reload values, when the current count decrements from two to zero the output of the flip-flop changes state; the current count will be reset to the reload value and counting will continue.

For odd reload values, the current count is always set to one less than the reload value.
If the output of the flip flop is low when the current count decrements from two to zero it will behave the same as the equivalent even reload value. However, if the output of the flip flop is high the reload will be delayed for one input signal cycle (0.8381 uS), which causes the "high" pulse to be slightly longer and the duty cycle will not be exactly 50%. Because the reload value is rounded down to the nearest even number anyway, it is recommended that only even reload values be used (which means you should mask the value before sending it to the port).

Note: This even value limitation on the reload value in mode 3 reduces the number of possible output frequencies in half. If you want to be able to control the frequency of IRQ0 to a somewhat higher degree, then think about using mode 2 instead for channel 0.

On channel 2, if the gate input goes low, counting stops and the output goes high immediately. Once the gate input has returned high, the next falling edge on input signal will cause the current count to be set to the reload value and operation will continue (with the output left high).

The reload value can be changed at any time, however the new value will not affect the current count until the current count is reloaded (when it is decreased from two to zero, or the gate input going low then high). When this occurs counting will continue using the new reload value.

A reload value (or divisor) of one must ''not'' be used with this mode.

=== Mode 4 – Software Triggered Strobe ===
Mode four operates as a retriggerable delay, and generates a pulse when the current count reaches zero.

When the mode/command register is written the output signal goes high and the PIT waits for the reload register to be set by software. After the reload register has been set, the current count will be set to the reload value on the next falling edge of the (1.193182 MHz) input signal. Subsequent falling edges of the input signal will decrement the current count (if the gate input is high on the preceding rising edge of the input signal).

When the current count decrements from one to zero, the output goes low for one cycle of the input signal (0.8381 uS). The current count will wrap around to 0xFFFF (or 0x9999 in BCD mode) and continue to decrement until the mode/command register or the reload register are set, however this will not affect the output state.

If the gate input goes low, counting stops but the output will not be affected and the current count will not be reset to the reload value.

The reload value can be changed at any time. When the new value has been set (both bytes for "lobyte/hibyte" access mode) it will be loaded into the current count on the next falling edge of the (1.193182 MHz) input signal, and counting will continue using the new reload value.

=== Mode 5 – Hardware Triggered Strobe ===
Mode 5 is similar to mode 4, except that it waits for the rising edge of the gate input to trigger (or re-trigger) the delay period (like mode 1). For this reason it is not usable for PIT channels 0 or 1 (where the gate input can't be changed).

When the mode/command register is written the output signal goes high and the PIT waits for the reload register to be set by software. After the reload register has been set the PIT will wait for the next rising edge of the gate input. Once this occurs, the current count will be set to the reload value on the next falling edge of the (1.193182 MHz) input signal. Subsequent falling edges of the input signal will decrement the current count.

When the current count decrements from one to zero, the output goes low for one cycle of the input signal (0.8381 uS). The current count will wrap around to 0xFFFF (or 0x9999 in BCD mode) and continue to decrement until the mode/command register or the reload register are set, however this will not affect the output state.

If the gate input signal goes low during this process it will have no effect. However, if the gate input goes high again it will cause the current count to be reloaded from the reload register on the next falling edge of the input signal, and restart the count again (the same as when counting first started).

The reload value can be changed at any time, however the new value will not affect the current count until the current count is reloaded (on the next rising edge of the gate input). When this occurs counting will continue using the new reload value.

== Counter Latch Command ==
To prevent the current count from being updated, it is possible to "latch" a PIT channel using the latch command. To do this, send the value CC000000 (in binary) to the mode/command register (I/O port 0x43), where 'CC' corresponds to the channel number. When the latch command has been sent, the current count is copied into an internal "latch register" which can then be read via the data port corresponding to the selected channel (I/O ports 0x40 to 0x42). The value kept in the latch register remains the same until it has been fully read, or until a new mode/command register is written.

The main benefit of the latch command is that it allows both bytes of the current count to be read without inconsistencies. For example, if you didn't use the latch command, then the current count may decrease from 0x0200 to 0x01FF after you've read the low byte but before you've read the high byte, so that your software thinks the counter was 0x0100 instead of 0x0200 (or 0x01FF).

While the latch command should not affect the current count, on some (old/dodgy) motherboards sending the latch command can cause a cycle of the input signal to be occasionally missed, which would cause the current count to be decremented 0.8381ms later than it should be. If you're sending the latch command often this could cause accuracy problems (but if you need to send the latch command often you may wish to consider redesigning your code anyway).

== Read Back Command ==
The read back command is a special command sent to the mode/command register (I/O port 0x43). The "read back" is not supported on the old 8253 chips but should be supported on all AT and later computers except for PS/2 (i.e. anything that isn't obsolete will support it).

For the read back command, the mode/command register uses the following format:

 Bits         Usage
 7 and 6      Must be set for the read back command
 5            Latch count flag (0 = latch count, 1 = don't latch count)
 4            Latch status flag (0 = latch status, 1 = don't latch status)
 3            Read back timer channel 2 (1 = yes, 0 = no)
 2            Read back timer channel 1 (1 = yes, 0 = no)
 1            Read back timer channel 0 (1 = yes, 0 = no)
 0            Reserved (should be clear)

Note: Be careful with bits 4 and 5 – they are inverted.

Bits 1 to 3 of the read back command select which PIT channels are affected, and allow multiple channels to be selected at the same time.

If bit 5 is clear, then any/all PIT channels selected with bits 1 to 3 will have their current count copied into their latch register (similar to sending the latch command, except it works for multiple channels with one command).

If bit 4 is clear, then for any/all PIT channels selected with bits 1 to 3, the next read of the corresponding data port will return a status byte (discussed below).

=== Read Back Status Byte ===

After sending a read back command with bit 4 clear, reading the data port for each selected channel will return a status value with the following format:

 Bit/s        Usage
 7            Output pin state
 6            Null count flags
 5 and 4      Access mode :
                 0 0 = Latch count value command
                 0 1 = Access mode: lobyte only
                 1 0 = Access mode: hibyte only
                 1 1 = Access mode: lobyte/hibyte
 3 to 1       Operating mode :
                 0 0 0 = Mode 0 (interrupt on terminal count)
                 0 0 1 = Mode 1 (hardware re-triggerable one-shot)
                 0 1 0 = Mode 2 (rate generator)
                 0 1 1 = Mode 3 (square wave generator)
                 1 0 0 = Mode 4 (software triggered strobe)
                 1 0 1 = Mode 5 (hardware triggered strobe)
                 1 1 0 = Mode 2 (rate generator, same as 010b)
                 1 1 1 = Mode 3 (square wave generator, same as 011b)
 0            BCD/Binary mode: 0 = 16-bit binary, 1 = four-digit BCD

The bottom six bits return the values that where programmed into the mode/command register when the channel was last initialized.

Bit 7 indicates the state of the PIT channel's output pin at the moment that the read-back command was issued.

Bit 6 indicates whether a newly-programmed divisor value has been loaded into the current count yet (if clear) or the channel is still waiting for a trigger signal or for the current count to count down to zero before a newly programmed reload value is loaded into the current count (if set). This bit is set when the mode/command register is initialized or when a new reload value is written, and cleared when the reload value is copied into the current count.

== Reading The Current Count ==
To read the current count using the "lobyte only" or "hibyte only" access modes, you can just do an "in al,0x40" (for PIT channel 0) without problems. For frequencies higher than 4.7 KHz it can be easiest to set the high byte of the reload value to zero, and then use the "lobyte only" access mode to minimize hassles.

For the "lobyte/hibyte" access mode you need to send the latch command (described above) to avoid getting wrong results. If any other code could try set the PIT channel's reload value or read its current count after you've sent the latch command but before you've read the highest 8 bits, then you have to prevent it. Disabling interrupts works for single CPU computers. For example, to read the count of PIT channel 0 you could use something like:

<syntaxhighlight lang="c">
unsigned read_pit_count(void) {
	unsigned count = 0;
	
	// Disable interrupts
	cli();
	
	// al = channel in bits 6 and 7, remaining bits clear
	outb(0x43,0b0000000);
	
	count = inb(0x40);		// Low byte
	count |= inb(0x40)<<8;		// High byte
	
	return count;
}
</syntaxhighlight>

== Setting The Reload Value ==
To set the reload value, just send the value/s to the corresponding data port. For the "lobyte only" or "hibyte only" access modes this only takes a single "out 0x40,al" (for PIT channel 0).

For the "lobyte/hibyte" access mode you need to send the low 8 bits followed by the high 8 bits. You must prevent other code from setting the PIT channel's reload value or reading its current count once you've sent the lowest 8 bits. Disabling interrupts works for single CPU computers. For example:

<syntaxhighlight lang="c">
void set_pit_count(unsigned count) {
	// Disable interrupts
	cli();
	
	// Set low byte
	outb(0x40,count&0xFF);		// Low byte
	outb(0x40,(count&0xFF00)>>8);	// High byte
	return;
}
</syntaxhighlight>

It should be noted that a reload value of zero can be used to specify a divisor of 65536. This is how the BIOS gets an IRQ 0 frequency as low as 18.2065 Hz. It should also be noted that setting the divisor to an exceptionally low value (resulting in an exceptionally high frequency) can freeze any and all programs, including the kernel. It is advised to keep the frequency between a reasonable range. For reference, newer versions of the Linux kernel defaulted to 1000hz (a divisor ≈1193.182).

== PIT Channel 0 Example Code ==
The following example code was written for NASM, but hasn't been tested.

The idea is to provide a single routine to initialize PIT channel 0 for any (possible) frequency and use IRQ 0 to accurately keep track of real time in milliseconds since the PIT was configured.

For the sake of accuracy, the initialization code will calculate the number of whole milliseconds to add to the "system timer tick" each IRQ, and the number of "fractions of a millisecond" to avoid drift. This may be important, for example if the PIT is set for 700 Hz it'd work out to (roughly) 1.42857 ms between IRQs, so keeping track of whole milliseconds only would lead to huge inaccuracies.

Hopefully, everyone is familiar with fixed point mathematics. For example, with the "32.32" notation I'll be using, if the high 32-bit value is equal to 0x00000001 and the low 32-bit value is equal to 0x80000000 then the combined value would be 1.5. In a similar way, the fraction 0.75 is represented with 0xC000000, 0.125 is represented with 0x20000000 and 0.12345 would be represented with 0x1F9A6B50.

To begin with, this following code contains all of the data used by this example. It is assumed that the ".bss" section is filled with zeros.

<syntaxhighlight lang="nasm">
section .bss
system_timer_fractions:  resd 1          ; Fractions of 1 ms since timer initialized
system_timer_ms:         resd 1          ; Number of whole ms since timer initialized
IRQ0_fractions:          resd 1          ; Fractions of 1 ms between IRQs
IRQ0_ms:                 resd 1          ; Number of whole ms between IRQs
IRQ0_frequency:          resd 1          ; Actual frequency of PIT
PIT_reload_value:        resw 1          ; Current PIT reload value
section .text
</syntaxhighlight>

Next is the handler for IRQ 0. It's fairly simple (all it does it add 2 64 bit fixed point values and send an EOI to the PIC chip).

<syntaxhighlight lang="nasm">
IRQ0_handler:
	push eax
	push ebx
 
	mov eax, [IRQ0_fractions]
	mov ebx, [IRQ0_ms]                    ; eax.ebx = amount of time between IRQs
	add [system_timer_fractions], eax     ; Update system timer tick fractions
	adc [system_timer_ms], ebx            ; Update system timer tick milli-seconds
 
	mov al, 0x20
	out 0x20, al                          ; Send the EOI to the PIC

	pop ebx
	pop eax
	iretd
</syntaxhighlight>

Now for the tricky bit – the initialization routine. The PIT can't generate some frequencies. For example if you want 8000 Hz then you've got a choice of 8007.93 Hz or 7954.544 Hz. In this case the following code will find the nearest possible frequency. Once it has calculated the nearest possible frequency it will reverse the calculation to find the actual frequency selected (rounded to the nearest integer, intended for display purposes only).

For some extra accuracy, I also use "3579545 / 3" instead of 1193182 Hz. This is mostly pointless due to inaccurate hardware (I just like being correct).

<syntaxhighlight lang="nasm">
 ;Input
 ; ebx   Desired PIT frequency in Hz
 
 init_PIT:
    pushad
 
    ; Do some checking
 
    mov eax,0x10000                   ;eax = reload value for slowest possible frequency (65536)
    cmp ebx,18                        ;Is the requested frequency too low?
    jbe .gotReloadValue               ; yes, use slowest possible frequency
 
    mov eax,1                         ;ax = reload value for fastest possible frequency (1)
    cmp ebx,1193181                   ;Is the requested frequency too high?
    jae .gotReloadValue               ; yes, use fastest possible frequency
 
    ; Calculate the reload value
 
    mov eax,3579545
    mov edx,0                         ;edx:eax = 3579545
    div ebx                           ;eax = 3579545 / frequency, edx = remainder
    cmp edx,3579545 / 2               ;Is the remainder more than half?
    jb .l1                            ; no, round down
    inc eax                           ; yes, round up
 .l1:
    mov ebx,3
    mov edx,0                         ;edx:eax = 3579545 * 256 / frequency
    div ebx                           ;eax = (3579545 * 256 / 3 * 256) / frequency
    cmp edx,3 / 2                     ;Is the remainder more than half?
    jb .l2                            ; no, round down
    inc eax                           ; yes, round up
 .l2:
 
 
 ; Store the reload value and calculate the actual frequency
 
 .gotReloadValue:
    push eax                          ;Store reload_value for later
    mov [PIT_reload_value],ax         ;Store the reload value for later
    mov ebx,eax                       ;ebx = reload value
 
    mov eax,3579545
    mov edx,0                         ;edx:eax = 3579545
    div ebx                           ;eax = 3579545 / reload_value, edx = remainder
    cmp edx,3579545 / 2               ;Is the remainder more than half?
    jb .l3                            ; no, round down
    inc eax                           ; yes, round up
 .l3:
    mov ebx,3
    mov edx,0                         ;edx:eax = 3579545 / reload_value
    div ebx                           ;eax = (3579545 / 3) / frequency
    cmp edx,3 / 2                     ;Is the remainder more than half?
    jb .l4                            ; no, round down
    inc eax                           ; yes, round up
 .l4:
    mov [IRQ0_frequency],eax          ;Store the actual frequency for displaying later
 
 
 ; Calculate the amount of time between IRQs in 32.32 fixed point
 ;
 ; Note: The basic formula is:
 ;           time in ms = reload_value / (3579545 / 3) * 1000
 ;       This can be rearranged in the following way:
 ;           time in ms = reload_value * 3000 / 3579545
 ;           time in ms = reload_value * 3000 / 3579545 * (2^42)/(2^42)
 ;           time in ms = reload_value * 3000 * (2^42) / 3579545 / (2^42)
 ;           time in ms * 2^32 = reload_value * 3000 * (2^42) / 3579545 / (2^42) * (2^32)
 ;           time in ms * 2^32 = reload_value * 3000 * (2^42) / 3579545 / (2^10)
 
    pop ebx                           ;ebx = reload_value
    mov eax,0xDBB3A062                ;eax = 3000 * (2^42) / 3579545
    mul ebx                           ;edx:eax = reload_value * 3000 * (2^42) / 3579545
    shrd eax,edx,10
    shr edx,10                        ;edx:eax = reload_value * 3000 * (2^42) / 3579545 / (2^10)
 
    mov [IRQ0_ms],edx                 ;Set whole ms between IRQs
    mov [IRQ0_fractions],eax          ;Set fractions of 1 ms between IRQs
 
 
 ; Program the PIT channel
 
    pushfd
    cli                               ;Disabled interrupts (just in case)
 
    mov al,00110100b                  ;channel 0, lobyte/hibyte, rate generator
    out 0x43, al
 
    mov ax,[PIT_reload_value]         ;ax = 16 bit reload value
    out 0x40,al                       ;Set low byte of PIT reload value
    mov al,ah                         ;ax = high 8 bits of reload value
    out 0x40,al                       ;Set high byte of PIT reload value
 
    popfd
 
    popad
    ret
</syntaxhighlight>

Note: you also need to install an IDT entry for IRQ 0, and unmask it in the PIC chip (or I/O APIC).

Of course it's easier to configure the PIT to a fixed value, but where's the fun in that? :-)

==Uses for the Timer IRQ==
===Using the IRQ to Implement <tt>sleep</tt>===
The PIT's generating a hardware interrupt every ''n'' milliseconds allows you to create a simple timer. Start with a global variable that contains the delay:
<syntaxhighlight lang="C">
volatile uint32_t CountDown;
</syntaxhighlight>
Next, every time the timer interrupt is called, decrement this variable until 0 is stored.
<syntaxhighlight lang="nasm">
 section .text
 global TimerIRQ
 TimerIRQ:
     push eax
     mov eax, [CountDown]
     test eax, eax
     jz TimerDone
     dec eax
     mov [CountDown], eax
 TimerDone:
     pop eax
     iretd
</syntaxhighlight>
Finally, create a function <tt>sleep</tt> that waits the interval, in milliseconds.
<syntaxhighlight lang="c">
void sleep(uint32_t millis) {
    CountDown = millis;
    while (CountDown > 0) {
        halt();
    }
}
</syntaxhighlight>

In a multitasking system, consider using a linked list or array of these CountDown variables. If your multitasking system supports interprocess communication, you can also store the semaphore/exchange where two processes can talk to, have the interrupt send a message to the waiting process when the timer is done, and have the waiting process block all execution until that message comes:

<syntaxhighlight lang="c">
#define COUNTDOWN_DONE_MSG 1
struct TimerBlock {
    EXCHANGE e;
    uint32_t CountDown;
} timerblocks[20];

void TimerIRQ(void) /* called from Assembly */
{
    uint8_t i;
    
    for (i = 0; i < 20; i++)
        if (timerblocks[i].CountDown > 0) {
            timerblocks[i].CountDown--;
            if (timerblocks[i].CountDown == 0)
                SendMessage(timerblocks[i].e, COUNTDOWN_DONE_MESSAGE);
        }
}

void Sleep(uint32_t delay)
{
    struct TimerBlock *t;

    if ((t = findTimerBlock()) == nil)
        return;
    t->CountDown = delay;
    WaitForMessageFrom(t->e = getCrntExch());
}
</syntaxhighlight>

In your documentation, note the interval of the timer. For example, if the timer interval is 10 milliseconds per tick, tell the programmer to issue

<syntaxhighlight lang="c">
Sleep(100);
</syntaxhighlight>

to sleep for a single second.

===Using the IRQ for Preemptive Multitasking===
The timer IRQ can also be used to perform preemptive multitasking. To give the currently running task some time to run, set a threshold, for example of 3 ticks. Use a global variable like the one before but go up from 0, and when that variable hits 3, switch tasks. How you do so is up to you.

== See Also ==
=== Articles ===
* [[Time And Date]]
* [[RTC]]
* [[HPET]]
* [[APIC Timer]]

=== Threads ===
=== External Links ===
* [[wikipedia:Programmable Interval Timer | Programmable Interval Timer]] on Wikipedia
* [http://www.osdever.net/bkerndev/Docs/pit.htm The PIT: A System Clock] on osdever

[[Category:Common Devices]]
[[Category:Timers]]
[[Category:X86]]
[[de:Programmable_Interval_Timer]]
````

<a id="w001561"></a>

## W001561 — Programmable interval timer

Bron-ID: W001561
Namespace: Main (0)
Revisie: 1993 | Datum: 2006-12-10T23:14:48Z
Bron: https://wiki.osdev.org/Programmable_interval_timer
Revisie-URL: https://wiki.osdev.org/index.php?title=Programmable_interval_timer&oldid=1993
Doorverwijzing: Programmable Interval Timer

````text
#REDIRECT [[Programmable Interval Timer]]
````

<a id="w001562"></a>

## W001562 — PIT

Bron-ID: W001562
Namespace: Main (0)
Revisie: 1994 | Datum: 2006-12-10T23:15:14Z
Bron: https://wiki.osdev.org/PIT
Revisie-URL: https://wiki.osdev.org/index.php?title=PIT&oldid=1994
Doorverwijzing: Programmable Interval Timer

````text
#REDIRECT [[Programmable Interval Timer]]
````

<a id="w001563"></a>

## W001563 — Category:Hardware

Bron-ID: W001563
Namespace: Category (14)
Revisie: 2325 | Datum: 2006-12-16T20:44:21Z
Bron: https://wiki.osdev.org/Category:Hardware
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AHardware&oldid=2325

````text
Articles about all kinds of hardware.

[[Category:Main Page Topics]]
````

<a id="w001564"></a>

## W001564 — DMI

Bron-ID: W001564
Namespace: Main (0)
Revisie: 28160 | Datum: 2023-07-10T14:45:15Z
Bron: https://wiki.osdev.org/DMI
Revisie-URL: https://wiki.osdev.org/index.php?title=DMI&oldid=28160

````text
{{Stub}}

DMI (Desktop Management Interface) is a standard that allows operating systems to collect basic information about the computer, and was developed by the Desktop Management Task Force (a group of over 120 vendors). Due to the rapid development of DMTF technologies, such as Common Information Model (CIM), the DMTF defined an "End of Life" process for DMI, which ended on March 31, 2005.

== See Also ==

=== Articles ===
* [[System Management BIOS]]
* [[BIOS]]

=== External Links ===
* [https://www.dmtf.org/sites/default/files/standards/documents/DSP0005.pdf Official documentation]
* [[wikipedia:Desktop_Management_Interface | Desktop Management Interface]] at Wikipedia.

[[Category:Standards]]
[[Category:Hardware Detection]]
````

<a id="w001565"></a>

## W001565 — Help:Editing

Bron-ID: W001565
Namespace: Help (12)
Revisie: 6618 | Datum: 2008-11-04T22:24:04Z
Bron: https://wiki.osdev.org/Help:Editing
Revisie-URL: https://wiki.osdev.org/index.php?title=Help%3AEditing&oldid=6618
Doorverwijzing: OSDev Wiki:Editing

````text
#REDIRECT [[OSDev Wiki:Editing]]
````

<a id="w001566"></a>

## W001566 — File:ISBN 0131429388.jpg

Bron-ID: W001566
Namespace: File (6)
Revisie: 13537 | Datum: 2012-06-13T02:40:58Z
Bron: https://wiki.osdev.org/File:ISBN_0131429388.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0131429388.jpg&oldid=13537

````text
Operating Systems Design and Implementation (3rd Edition)
[[Category:Book Covers]]
````

<a id="w001567"></a>

## W001567 — OSDev Wiki talk:Pages To Convert/Questionable pages

Bron-ID: W001567
Namespace: OSDev Wiki talk (5)
Revisie: 2006 | Datum: 2006-12-11T01:57:56Z
Bron: https://wiki.osdev.org/OSDev_Wiki_talk:Pages_To_Convert/Questionable_pages
Revisie-URL: https://wiki.osdev.org/index.php?title=OSDev_Wiki_talk%3APages_To_Convert%2FQuestionable_pages&oldid=2006

````text
=== Pages about Users ===
Articles about users (not user pages) should exist under the main Wiki, but they should only exist if that person is creditable within the osdev comunity. Creation of autobiographies should be discouraged, although a person may correct information written regarding themselves. And information posted on their page should be relevant to OSDev'ing. Their personal hobbies and lives should be restricted to their user page, not their article. This is a slight variation on Wikipedia's method of biographies.

--[[User:MessiahAndrw|MessiahAndrw]] 19:57, 10 December 2006 (CST)
````

<a id="w001568"></a>

## W001568 — Template:Book

Bron-ID: W001568
Namespace: Template (10)
Revisie: 24238 | Datum: 2020-01-03T03:34:51Z
Bron: https://wiki.osdev.org/Template:Book
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ABook&oldid=24238

````text
{| 
|-
| rowspan="6" valign="top" align="center" width="200" | [[Image:ISBN_{{{ISBN}}}.jpg|120px]]<br />[http://www.amazon.com/exec/obidos/ASIN/{{{ASIN}}}/osdev-20/ http://wiki.osdev.org/images/3/3a/Buy_from_amazon.gif]
| height="50" | <font size="+1">{{{title}}}</font>
|-
| valign="top" height="10" | '''Author(s):''' {{{author}}}
|-
| valign="top" height="10" | '''Year:''' {{{year|}}}
|-
| valign="top" height="10" | '''Supplementary material and/or errata:''' {{{supp}}}
|-
| valign="top" | '''Official Description:''' {{{description}}}
|-
| valign="top" | '''Review:''' {{{review}}}
|}
<br />
<br />
````

<a id="w001570"></a>

## W001570 — File:Buy from amazon.gif

Bron-ID: W001570
Namespace: File (6)
Revisie: 2012 | Datum: 2006-12-11T02:31:42Z
Bron: https://wiki.osdev.org/File:Buy_from_amazon.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ABuy_from_amazon.gif&oldid=2012

````text

````

<a id="w001571"></a>

## W001571 — File:ISBN 0201596164.jpg

Bron-ID: W001571
Namespace: File (6)
Revisie: 2040 | Datum: 2006-12-11T05:08:46Z
Bron: https://wiki.osdev.org/File:ISBN_0201596164.jpg
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AISBN_0201596164.jpg&oldid=2040

````text
The Indispensable PC Hardware Book (4th Edition)
[[Category:Book Covers]]
````

<a id="w001572"></a>

## W001572 — Books

Bron-ID: W001572
Namespace: Main (0)
Revisie: 29661 | Datum: 2025-09-18T03:57:46Z
Bron: https://wiki.osdev.org/Books
Revisie-URL: https://wiki.osdev.org/index.php?title=Books&oldid=29661

````text
__NOEDITSECTION__

= Algorithms =

{{Book|ISBN=0073523402|ASIN=0073523402
|title=Algorithms
|author=Sanjoy Dasgupta, Christos Papadimitriou, Umesh Vazirani
|year=2006
|supp=[http://www.cs.berkeley.edu/~vazirani/algorithms.html Penultimate draft], [http://cseweb.ucsd.edu/~dasgupta/book/index.html Errata], [http://highered.mcgraw-hill.com/sites/0073523402/ Link]
|description=This text, extensively class-tested over a decade at UC Berkeley and UC San Diego, explains the fundamentals of algorithms in a story line that makes the material enjoyable and easy to digest.

Emphasis is placed on understanding the crisp mathematical idea behind each algorithm, in a manner that is intuitive and rigorous without being unduly formal.

Features include:

* The use of boxes to strengthen the narrative: pieces that provide historical context, descriptions of how the algorithms are used in practice, and excursions for the mathematically sophisticated.
* Carefully chosen advanced topics which can be skipped in a standard one-semester course, but can be covered in an advanced algorithms course or in a more leisurely two-semester sequence.
* An accessible treatment of linear programming introduces students to one of the greatest achievements in algorithms. An optional chapter on the quantum algorithm for factoring provides an unique peephole into this exciting topic.
|review=}}

{{Book|ISBN=0130220051|ASIN=0130220051
|title=Algorithms and Data Structures|author=Niklaus Wirth
|year=1985
|supp=[http://www.inf.ethz.ch/personal/wirth/AD.pdf Oberon Version 2004]
|description=The classical textbook on algorithms and data structures. Citing the amazon review of "oversteer": The book goes over practically important algorithms and data structures. It may not include the latest and greatest advances, but it is still very relevant in today's practical applications.
To me, the highest value of the book is not the particular methods explained, but the good style and the way of thinking about programs that the book teaches. There are very few books that can actually teach good style, and this is probably one of the best.
This book is a must read for anyone wishing to become a great programmer, not merely an average one.|review=[https://en.wikipedia.org/wiki/Algorithms_%2B_Data_Structures_%3D_Programs Wikipedia]}}

{{Book|ISBN=032157351X|ASIN=032157351X
|title=Algorithms, 4th ed.
|author=Kevin Wayne, and Robert Sedgewick
|year=2011
|supp=[http://algs4.cs.princeton.edu/ Link]
|description=The textbook ''Algorithms, 4th Edition'' by Robert Sedgewick and Kevin Wayne [ [http://www.amazon.com/Algorithms-4th-Robert-Sedgewick/dp/032157351X Amazon] &middot; [http://www.pearsonhighered.com/educator/product/Algorithms/9780321573513.page Pearson] &middot; [http://www.informit.com/store/product.aspx?isbn=032157351X InformIT] ] surveys the most important algorithms and data structures in use today. The textbook is organized into six chapters:

* ''Chapter 1: Fundamentals'' introduces a scientific and engineering basis for comparing algorithms and making predictions. It also includes our programming model.
* ''Chapter 2: Sorting'' considers several classic sorting algorithms, including insertion sort, mergesort, and quicksort. It also includes a binary heap implementation of a priority queue.
* ''Chapter 3: Searching'' describes several classic symbol table implementations, including binary search trees, red-black trees, and hash tables.
* ''Chapter 4: Graphs'' surveys the most important graph processing problems, including depth-first search, breadth-first search, minimum spanning trees, and shortest paths.
* ''Chapter 5: Strings'' investigates specialized algorithms for string processing, including radix sorting, substring search, tries, regular expressions, and data compression.
* ''Chapter 6: Context'' highlights connections to systems programming, scientific computing, commercial applications, operations research, and intractability.

Applications to science, engineering, and industry are a key feature of the text. We motivate each algorithm that we address by examining its impact on specific applications.
|review=}}

{| 
|-
| rowspan="6" valign="top" align="center" width="200" |
[[Image:ISBN_0321751043.jpg|120px]]<br/>[http://www.amazon.com/exec/obidos/ASIN/0321751043/osdev-20/ http://wiki.osdev.org/images/3/3a/Buy_from_amazon.gif]<br/><br/>
[[Image:ISBN_0201853922.jpg|120px]]<br/>[http://www.amazon.com/exec/obidos/ASIN/0201853922/osdev-20/ http://wiki.osdev.org/images/3/3a/Buy_from_amazon.gif]
| height="50" | <font size="+1">The Art of Computer Programming</font>
|-
| valign="top" height="10" | '''Author(s):''' Donald E. Knuth
|-
| valign="top" height="10" | '''Year:''' 2011
|-
| valign="top" height="10" | '''Supplementary material and/or errata:''' [http://www-cs-faculty.stanford.edu/~uno/taocp.html Link]
|-
| valign="top" | '''Official Description:''' At the end of 1999, these books were named among the best twelve physical-science monographs of the century by ''[http://www.americanscientist.org/bookshelf/pub/100-or-so-books-that-shaped-a-century-of-science American Scientist]'' , along with: Dirac on quantum mechanics, Einstein on relativity, Mandelbrot on fractals, Pauling on the chemical bond, Russell and Whitehead on foundations of mathematics, von Neumann and Morgenstern on game theory, Wiener on cybernetics, Woodward and Hoffmann on orbital symmetry, Feynman on quantum electrodynamics, Smith on the search for structure, and Einstein's collected papers. Wow!
|-
| valign="top" | '''Review:''' 
|}

{{Book|ISBN=0262033844|ASIN=0262033844
|title=Introduction to Algorithms, 3rd ed.
|author=Thomas H. Cormen, Charles E. Leiserson, Ronald L Rivest, Clifford Stein
|year=2009
|supp=[http://mitpress.mit.edu/algorithms/ Link]
|description=Some books on algorithms are rigorous but incomplete; others cover masses of material but lack rigor. ''Introduction to Algorithms'' uniquely combines rigor and comprehensiveness. The book covers a broad range of algorithms in depth, yet makes their design and analysis accessible to all levels of readers. Each chapter is relatively self-contained and can be used as a unit of study. The algorithms are described in English and in a pseudocode designed to be readable by anyone who has done a little programming. The explanations have been kept elementary without sacrificing depth of coverage or mathematical rigor.

The first edition became a widely used text in universities worldwide as well as the standard reference for professionals. The second edition featured new chapters on the role of algorithms, probabilistic analysis and randomized algorithms, and linear programming. The third edition has been revised and updated throughout. It includes two completely new chapters, on van Emde Boas trees and multithreaded algorithms, and substantial additions to the chapter on recurrences (now called "Divide-and-Conquer"). It features improved treatment of dynamic programming and greedy algorithms and a new notion of edge-based flow in the material on flow networks. Many new exercises and problems have been added for this edition.

As of the third edition, this textbook is published exclusively by the MIT Press.
|review=}}

= Compiler Theory =

{{Book|ISBN=1558603204|ASIN=1558603204
|title=Advanced Compiler Design and Implementation
|author=Steven S. Muchnick
|year=1997
|supp=https://web.archive.org/web/20250626152135/https://www.math.tau.ac.il/~msagiv/courses/acd/bugs.htm
|description=The definitive book on advanced compiler design.

This comprehensive, up-to-date work examines advanced issues in the design
and implementation of compilers for modern processors. Written for
professionals and graduate students, the book guides readers in designing
and implementing efficient structures for highly optimizing compilers for
real-world languages. Covering advanced issues in fundamental areas of
compiler design, this book discusses a wide array of possible code
optimizations, determining the relative importance of optimizations, and
selecting the most effective methods of implementation.

* Lays the foundation for understanding the major issues of advanced compiler design.

* Treats optimization in-depth.

* Uses four case studies of commercial compiling suites to illustrate different approaches to compiler structure, intermediate-code design, and optimization―these include Sun Microsystems's compiler for SPARC, IBM's for POWER and PowerPC, DEC's for Alpha, and Intel's for Pentium an related processors.

* Presents numerous clearly defined algorithms based on actual cases.

* Introduces Informal Compiler Algorithm Notation (ICAN), a language devised by the author to communicate algorithms effectively to people.
|review=}}

{{Book|ISBN=0130525642|ASIN=0130525642
|title=Assemblers and Loaders
|author=David Salomon
|year=1993
|supp=[http://www.davidsalomon.name/assem.advertis/AssemAd.html Freely available]
|description=Written during 1985--1992, the book developed from class notes on computer organization. It includes numerous exercises with answers provided, and review questions following each chapter. The book is bound as a paperback with a colorful cover.
|review=}}

{{Book|ISBN=0321486811|ASIN=0321486811
|title=Compilers: Principles, Techniques, & Tools
|year=2006
|author=Alfred V. Aho, Monica S. Lam, Ravi Sethi, and Jeffrey D. Ullman
|supp=[http://dragonbook.stanford.edu/ Link]
|description=''Compilers: Principles, Techniques and Tools'', known to professors, students, and developers worldwide as the "Dragon Book," is available in a new edition.  Every chapter has been completely revised to reflect developments in software engineering, programming languages, and computer architecture that have occurred since 1986, when the last edition published.  The authors, recognizing that few readers will ever go on to construct a compiler, retain their focus on the broader set of problems faced in software design and software development.

New chapters include:  
 
Chapter 10 Instruction-Level Parallelism<br/>
Chapter 11 Optimizing for Parallelism and Locality<br/>
Chapter 12 Interprocedural Analysis
|review=}}

{{Book|ISBN=9780128154120|ASIN=0128154128
|title=Engineering a Compiler
|year=2022
|supp=
|author=Keith D. Cooper, Linda Torczon
|description=Engineering a Compiler, Third Edition covers the latest developments in compiler technology, with new chapters focusing on semantic elaboration (the problems that arise in generating code from the ad-hoc syntax-directed translation schemes in a generated parser), on runtime support for naming and addressability, and on code shape for expressions, assignments and control-structures. Leading educators and researchers, Keith Cooper and Linda Torczon, have revised this popular text with a fresh approach to learning important techniques for constructing a modern compiler, combining basic principles with pragmatic insights from their own experience building state-of-the-art compilers.

* Presents in-depth treatments of algorithms and techniques used in the front end of a modern compiler
* Pays particular attention to code optimization and code generation, both primary areas of recent research and development
* Focuses on how compilers (and interpreters) implement abstraction, tying the underlying knowledge to students’ own experience and to the languages in which they have been taught to program
* Covers bottom-up methods of register allocation at the local scope
|review=}}

{{Book|ISBN=1558604960|ASIN=1558604960
|title=Linkers and Loaders
|author=John R. Levine
|year=1999
|supp=[http://www.iecc.com/linker/ Free manuscript], [http://linker.iecc.com/ Errata and code]
|description=Whatever your programming language, whatever your platform, you probably tap into linker and loader functions all the time. But do you know how to use them to their greatest possible advantage? Only now, with the publication of Linkers and Loaders, is there an authoritative book devoted entirely to these deep-seated compile-time and run-time processes. The book begins with a detailed and comparative account of linking and loading that illustrates the differences among various compilers and operating systems. On top of this foundation, the author presents clear practical advice to help you create faster, cleaner code. You'll learn to avoid the pitfalls associated with Windows DLLs, take advantage of the space-saving, performance-improving techniques supported by many modern linkers, make the best use of the UNIX ELF library scheme, and much more. If you're serious about programming, you'll devour this unique guide to one of the field's least understood topics. Linkers & Loaders is also an ideal supplementary text for compiler and operating systems courses.

'''Features'''

* Includes a linker construction project written in Perl, with project files available for download.
* Covers dynamic linking in Windows, UNIX, Linux, BeOS, and other operating systems.
* Explains the Java linking model and how it figures in network applets and extensible Java code.
* Helps you write more elegant and effective code, and build applications that compile, load, and run more efficiently.
|review=Excellently written and easy to read, yet thorough and informative. It has just enough whimsy to make what most readers find a dry topic entertaining. While the current edition is now somewhat dated, it covers the majority of what one needs to understand how both linkage editors and program loaders work. - [[User:Schol-r-lea]]}}

= Computer Architecture =

== Theory ==

{{Book|ISBN=012383872X|ASIN=012383872X
|title=Computer Architecture: A Quantitative Approach, 5th ed.
|author=John L. Hennessy, and David A. Patterson
|year=2011
|supp=[http://booksite.mkp.com/9780123838728/index.php Link]
|description=The computing world today is in the middle of a revolution: mobile clients and cloud computing have emerged as the dominant paradigms driving programming and hardware innovation today. The Fifth Edition of Computer Architecture focuses on this dramatic shift, exploring the ways in which software and technology in the cloud are accessed by cell phones, tablets, laptops, and other mobile computing devices. Each chapter includes two real-world examples, one mobile and one datacenter, to illustrate this revolutionary change.
|review=}}

{{Book|ISBN=0136108040|ASIN=0136108040
|title=Computer Systems: A Programmer's Perspective, 2nd ed.
|author=Randal E. Bryant, and David R. O'Hallaron
|year=2010
|supp=[http://csapp.cs.cmu.edu/ Manuscript, Web asides, errata], [http://www.pearsonhighered.com/pearsonhigheredus/educator/product/products_detail.page?isbn=9780136108047 Link], [http://csappbook.blogspot.ro/ Blog]
|description= This book (CS:APP2e) is the second edition of a book that stems from the introductory computer systems course we developed at Carnegie Mellon University, starting in the Fall of 1998, called "Introduction to Computer Systems" (ICS). The presentation is based on the following principles, which aim to help the students become better programmers and to help prepare them for upper-level systems courses:

* Students should be introduced to computer systems from the perspective of a programmer, rather from the more traditional perspective of a system implementer. [http://csapp.cs.cmu.edu/public/perspective.html What does this mean?]
* Students should get a view of the complete system, comprising the hardware, operating system, compiler, and network.
* Students learn best by developing and evaluating real programs that run on real machines.

We cover data representations, machine level representations of C programs, processor architecture, program optimizations, the memory hierarchy, linking, exceptional control flow (exceptions, interrupts, processes, and Unix signals), virtual memory and memory management, system-level I/O, basic network programming, and concurrent programming. These concepts are supported by series of fun and hands-on lab assignments. See the manuscript [http://csapp.cs.cmu.edu/public/pieces/preface.pdf Preface] for more details.
|review=}}

{{Book|ISBN=0132916525|ASIN=0132916525
|title=Structured Computer Organization, 6th ed.
|author=Andrew S. Tanenbaum, and Todd Austin
|year=2012
|supp=[http://www.pearsonhighered.com/educator/product/Structured-Computer-Organization/9780132916523.page Link]
|description=''Structured Computer Organization,'' specifically written for undergraduate students, is a best-selling guide that provides an accessible introduction to computer hardware and architecture. This text will also serve as a useful resource for all computer professionals and engineers who need an overview or introduction to computer architecture. 

This book takes a modern structured, layered approach to understanding computer systems. It's highly accessible - and it's been thoroughly updated to reflect today's most critical new technologies and the latest developments in computer organization and architecture. Tanenbaum’s renowned writing style and painstaking research make this one of the most accessible and accurate books available, maintaining the author’s popular method of presenting a computer as a series of layers, each one built upon the ones below it, and understandable as a separate entity.
|review=}}

== Implementations ==

=== PC ===

{{Book|ISBN=0201596164|ASIN=0201596164
|title=The Indispensable PC Hardware Book, 4th ed.
|author=Hans-Peter Messmer
|year=2001
|supp=N/A
|description=This fully revised and popular book is now up-to-date and even more comprehensive than before. '''The Indispensable PC Hardware Book 4/e''' will be 'indispensable' to anyone who wants to know more about the inner workings of a personal computer: from programmers who want access to hardware components; professionals and home-users who wants to or has to understand the structure and functioning of a personal computer; to users who want to up-grade their PC's, and dealers who wish to advise their customers--this book will provide the solution to all your hardware questions. Even beginners should not shy away as this book begins with an easy introduction to the subject area.

Key highlights:

* CPU's from the 8086/8088 to the Pentium III and Athlon
* Real, protected and virtual models
* Windows and plug&play devices
* CPU Clones from all major manufacturers
* Chipsets and support chips
* Timers, interrupts and DMA
* I/O programming and PCI bus programming
* AGP variants and graphic systems
* Universal serial bus
* Local storage from the diskette to DVD
* Memory systems, SDRAM, EDO, flas, RAM bus, and modules
* Extensive glossary which explains most of the terms and concepts related to personal computer hardware.
* Appendices brimming with practical advice, especially for programmers.
|review=}}

{{Book|ISBN=0201479508|ASIN=0201479508
|title=The Undocumented PC, 2nd ed.
|author=Frank van Gilluwe
|year=1996
|supp=N/A
|description='''The Undocumented PC''' is your technical guide into the inner workings of the PC. At over 1100 pages, '''The Undocumented PC''' is the standard that programmers around the world rely on to create powerful programs and to better understand all areas of the PC. Get the competitive advantage of knowing how to utilize and connect to PC firmware and hardware.
|review=}}

= Human-Computer Interaction =



= Networking =

== Theory ==

{{Book|ISBN=0132126958|ASIN=0132126958
|title=Computer Networks, 5th ed.
|author=Andrew S. Tanenbaum, and David J. Wetherall
|year=2010
|supp=[http://www.computernetworks5e.org/ Link]
|description=''Computer Networks, 5/e'' is appropriate for Computer Networking or Introduction to Networking courses at both the undergraduate and graduate level in Computer Science, Electrical Engineering, CIS, MIS, and Business Departments.

Tanenbaum takes a structured approach to explaining how networks work from the inside out. He starts with an explanation of the physical layer of networking, computer hardware and transmission systems; then works his way up to network applications. Tanenbaum's in-depth application coverage includes email; the domain name system; the World Wide Web (both client- and server-side); and multimedia (including voice over IP, Internet radio video on demand, video conferencing, and streaming media. Each chapter follows a consistent approach: Tanenbaum presents key principles, then illustrates them utilizing real-world example networks that run through the entire book&mdash;the Internet, and wireless networks, including Wireless LANs, broadband wireless and Bluetooth. The Fifth Edition includes a chapter devoted exclusively to network security. The textbook is supplemented by a Solutions Manual, as well as a Website containing PowerPoint slides, art in various forms, and other tools for instruction, including a protocol simulator whereby students can develop and test their own network protocols.
|review=}}

{{Book|ISBN=0132856204|ASIN=0132856204
|title=Computer Networking: A Top-Down Approach, 6th ed.
|author=James F. Kurose, Keith W. Ross
|year=2012
|supp=[http://www.pearsonhighered.com/product?isbn=0132856204 Link]
|description=Computer Networking continues with an early emphasis on application-layer paradigms and application programming interfaces (the top layer), encouraging a hands-on experience with protocols and networking concepts, before working down the protocol stack to more abstract layers.

This book has become the dominant book for this course because of the authors’ reputations, the precision of explanation, the quality of the art program, and the value of their own supplements.

Visit the authors’ blog for information and resources to discuss the newest edition, as well as valuable insights, teaching tips, and discussion about the field of Computer Networking http://kuroseross.com
|review=}}

== Implementations ==

=== AppleTalk ===

{{Book|ISBN=0201550210|ASIN=0201550210
|title=Inside AppleTalk, 2nd ed.
|year=1990
|author=Gursharan S. Sidhu, Richard F. Andrews, and Alan B. Oppenheimer
|supp=[http://web.archive.org/web/20011124183351/http://developer.apple.com/macos/opentransport/docs/dev/Inside_AppleTalk.pdf Free book]
|description=Written by Gursharan S. Sidhu, technical director of Network Systems Development at Apple Computer, and two of Apple's senior staff engineers - Richard F. Andrews and Alan B. Oppenheimer - "Inside AppleTalk" provides an in-depth discussion of the protocol architecture of the AppleTalk network system. This Second Edition features completely updated, detailed descriptions of the AppleTalk protocols, including the enhancements of AppleTalk Phase 2, as well as a fundamental overview by Mr. Sidhu.
|review=}}

=== OSI ===

{{Book|ISBN=0136832105|ASIN=0136832105
|title=The Little Black Book: Mail Bonding with OSI Directory Services
|author=Marshall T. Rose
|year=1991
|supp=N/A
|description=From the author of the bestselling The Open Book and The Simple Book, this comprehensive guide to directory services technology provides the underlying infrastructure to a variety of communication services. The guide explores protocols and protocol suites, giving the reader techniques on simplified message handling and routing over electronic networks. It discusses the network model message handling, user agents, message transfer, gateway, and more used in electronic communications. The handbook introduces such topics as X.500 directory, and will be of interest to those involved with electronic mail, network communications, and other electronic networks.
|review=}}

{{Book|ISBN=0136430163|ASIN=0136430163
|title=The Open Book: A Practical Perspective on OSI
|author=Marshall T. Rose
|year=1990
|supp=N/A
|description=The OSI protocol suite (an internationally standardized set of rules for computer communications) offers a unique promise: it has the potential to provide a political and technical solution to world-wide networking. Computers--from supers to workstations and PCs, from the executive suite to the laboratory, and from Palo Alto to London, or anywhere--can all share a common set of rules for communicating. But, with any emerging technology, there are numerous questions to be answered and issues to be addressed. The Open Book is designed to examine those complex questions and issues and provide a balanced set of perspectives.
|review=}}

=== TCP/IP ===

{| 
|-
| rowspan="6" valign="top" align="center" width="200" |
[[Image:ISBN_0321336313.jpg|120px]]<br/>[http://www.amazon.com/exec/obidos/ASIN/0321336313/osdev-20/ http://wiki.osdev.org/images/3/3a/Buy_from_amazon.gif]<br/><br/>
[[Image:ISBN_020163354X.jpg|120px]]<br/>[http://www.amazon.com/exec/obidos/ASIN/020163354X/osdev-20/ http://wiki.osdev.org/images/3/3a/Buy_from_amazon.gif]<br/><br/>
[[Image:ISBN_0201634953.jpg|120px]]<br/>[http://www.amazon.com/exec/obidos/ASIN/0201634953/osdev-20/ http://wiki.osdev.org/images/3/3a/Buy_from_amazon.gif]
| height="50" | <font size="+1">TCP/IP Illustrated</font>
|-
| valign="top" height="10" | '''Author(s):''' Gary R. Wright, Kevin R. Fall, and W. Richard Stevens
|-
| valign="top" height="10" | '''Year:''' 2011 (Vol1), 1995 (Vol2), 1996 (Vol3)
|-
| valign="top" height="10" | '''Supplementary material and/or errata:''' [http://www.kohala.com/start/unpv22e/unpv22e.html Volume 2], [http://www.kohala.com/start/tcpipiv3.html Volume 3]
|-
| valign="top" | '''Official Description:''' The '''TCP/IP Illustrated''' books are praised for their highly effective visual approach to the essential TCP/IP topics facing today's networking professionals. The word 'illustrated' distinguishes this book from the rest. By forcing conditions to occur, and then displaying the results, '''TCP/IP Illustrated''' gives readers a much greater understanding of the concepts than words alone can provide. The books are noted for their diagrams and clear and readable writing style.
|-
| valign="top" | '''Review:''' 
|}

= Peripherals =

== Implementations ==

=== USB ===

{{Book|ISBN=1717425364|ASIN=1717425364
|title=USB: The Universal Serial Bus
|author=Benjamin David Lunt
|year=2018
|supp=[http://www.fysnet.net/the_universal_serial_bus.htm Link]
|description='''This book is for those who are building an operating system, writing embedded code, or just want to learn how to communicate with the USB hardware and attached devices directly through hardware programming.'''

This book will show you how to find and initialize each of the four major USB Host Controllers, set up USB stacks, and communicate with attached devices.
It has detailed information and examples of each step required to accomplish this. Detailed information for the UHCI, OHCI, EHCI, and the new Super Speed xHCI controller is easy to follow and understand.

'''Resources Included:'''
* Source code - Shows how to detect a controller and send/receive data to/from the attached device(s).
|review=https://www.amazon.com/dp/1717425364/#customerReviews}}


= Operating Systems =

== Theory ==

{{Book|ISBN=0136006639|ASIN=0136006639
|title=Modern Operating Systems, 3rd ed.
|author=Andrew S. Tanenbaum
|year=2007
|supp=[http://www.pearsonhighered.com/pearsonhigheredus/educator/product/products_detail.page?isbn=0136006639 Link]
|description='''For Introductory Courses in Operating Systems in Computer Science, Computer Engineering, and Electrical Engineering programs.'''

The widely anticipated revision of this worldwide best-seller incorporates the latest developments in operating systems (OS)technologies. The Third Edition includes up-to-date materials on relevant OS such as Linux, Windows, and embedded real-time and multimedia systems. Tanenbaum also provides information on current research based on his experience as an operating systems researcher.

'''Student Resources Include:'''
* Online Exercises - Provide hands-on experience with building as well as analyzing the performance of OS. In particular, these exercises have been designed to provide experience with analyzing the resource consumptions in Windows and Linux.
* Simulation Exercises - Designed to provide experience with building some key components of an OS, including process scheduling, main memory allocation, paging algorithms and virtual memory, and file systems.
* Lab Experiments
* Please note, GOAL is no longer available with this book.

'''Password-Protected Instructor Resources (Select the Resources Tab to View Downloadable Files):'''
* Power Point Lecture Slides
* Figures in both .jpeg and .eps file format
* Solutions to Exercises
* Please note, GOAL is no longer available with this book.

''Modern Operating Systems, 3e'' is the recipient of the [http://taaonline.net/index.html Text and Authors Association (TAA)] '''2010 McGuffey Longevity Award.''' The McGuffey Longevity Award recognizes textbooks whose excellence has been demonstrated over time.
|review=}}

{{Book|ISBN=1118063333|ASIN=1118063333
|title=Operating System Concepts, 9th ed.
|author=Avi Silberschatz, Peter Baer Galvin, and Greg Gagne
|year=2012
|supp=[http://codex.cs.yale.edu/avi/os-book/OS9/os9c/index.html Link]
|description=
|review=}}

{{Book|ISBN=013230998X|ASIN=013230998X
|title=Operating systems: Internals and Design Principles, 7th ed.
|author=William Stallings
|year=2011
|supp=[http://williamstallings.com/OperatingSystems/ Link 1], [http://www.pearsonhighered.com/educator/product/Operating-Systems-Internals-and-Design-Principles/9780132309981.page Link 2]
|description=A state-of-the art survey of operating system principles. Covers fundamental technology as well as contemporary design issues, such as threads, microkernels, SMPs, real-time systems, multiprocessor scheduling, embedded OSs, distributed systems, clusters, security, and object-oriented design. '''Third and fourth editions received the TAA award for the best Computer Science and Engineering Textbook of the year.'''
|review=}}

{{Book|ISBN=198508659X|ASIN=B00TPZ17O4
|title=Operating Systems: Three Easy Pieces
|author=Remzi H. Arpaci-Dusseau and Andrea C. Arpaci-Dusseau 
|year=2015
|supp=[http://pages.cs.wisc.edu/~remzi/OSTEP/ Website]
|description=A free book discussing operating system theory an implementation. The book is centered around three conceptual pieces that are fundamental to operating systems: virtualization, concurrency, and persistence. In understanding the conceptual, you will also learn the practical, including how an operating system does things like schedule the CPU, manage memory, and store files persistently.
|review=}}

== Implementations ==

{{Book|ISBN=1588530000|ASIN=1588530000
|title=MMURTL V1.0
|author=Richard A. Burgess
|year=2000
|supp=[http://www.ipdatacorp.com/mmurtl/ Book, code]
|description='''MMURTL-V1.0''' - is a book that is very popular among computer programmers interested in the internal workings of a computer operating system. Shown to the left as originally printed, it was previously titled '''''"Developing Your own 32 Bit Operating System"''' (ISBN 0-672-30655-7)'' and was published by SAMS/MacMillan. We have renamed the book '''''MMURTL-V1.0''' (ISBN 1-58853-000-0)'' and the printed version is a "Perfect Bind" soft cover (see the additional picture at very bottom of page).  It was renamed because this is the name that the author had originally intended. '''MMURTL''' is a ''bad'' acronym for '''M'''essage based '''MU'''ltitasking '''R'''eal '''T'''ime kerne'''L''' (to quote the author: '''''"Hey.. all the good acronyms were taken!"''''').

This is literally a "how to book" on writing a computer operating system, and includes the assembler and C language code to build one.  In fact, it contains the complete operating system source code, an assembler and C compiler, along with their source code, on the accompanying CD! This is the exact text of the original book except where the author has made very minor corrections where the original text was incorrect (editing mistakes, typos, etc.). You may view the first chapter which is an overview of the book, and the author's goals with the link in the table below.

The PDF ebook is approximately '''2.7 Mb'''. Total size of all files to download is approximately '''8.5 Mb.''' The printed version is approximately 604 pages, while the PDF version is 667 pages. The printed version also contains a complete index generated by the author.
|review=}}

{{Book|ISBN=0131429388|ASIN=0131429388
|title=Operating Systems: Design and Implementation, 3rd ed.
|author=Andrew S. Tanenbaum, and Albert S. Woodhull
|year=2006
|supp=[http://www.pearsonhighered.com/educator/academic/product/0,,0131429388,00%2ben-USS_01DBC.html Link]
|description='''''Operating Systems Design and Implementation, 3e''''', is ideal for introductory courses on computer operating systems. Written by the creator of Minix, professional programmers will now have the most up-to-date tutorial and reference available today.
 
Revised to address the latest version of MINIX (MINIX 3), this streamlined, simplified new edition remains the only operating systems text to first explain relevant principles, then demonstrate their applications using a Unix-like operating system as a detailed example. It has been especially designed for high reliability, for use in embedded systems, and for ease of teaching.
|review=}}

{{Book|ISBN=1558604979|ASIN=1558604979
|title=Practical File System Design with the Be File System
|author=Dominic Giampaolo
|year=1998
|supp=[http://www.nobius.org/~dbg/ Free book and code]
|description=This is the new guide to the design and implementation of file systems in general, and the Be File System (BFS) in particular. This book covers all topics related to file systems, going into considerable depth where traditional operating systems books often stop. Advanced topics are covered in detail such as journaling, attributes, indexing and query processing. Built from scratch as a modern 64 bit, journaled file system, BFS is the primary file system for the Be Operating System (BeOS), which was designed for high performance multimedia applications. 

You do not have to be a kernel architect or file system engineer to use Practical File System Design. Neither do you have to be a BeOS developer or user. Only basic knowledge of C is required. If you have ever wondered about how file systems work, how to implement one, or want to learn more about the Be File System, this book is all you will need.

* Review of other file systems, including Linux ext2, BSD FFS, Macintosh HFS, NTFS and SGI's XFS.
* Allocation policies for placing data on disks and discussion of on-disk data structures used by BFS 
* How to implement journaling
* How a disk cache works, including cache interactions with the file system journal
* File system performance tuning and benchmarks comparing BFS, NTFS, XFS, and ext2
* A file system construction kit that allows the user to experiment and create their own file systems
|review=}}

{{Book|ISBN=0201544288|ASIN=0201544288
|title=Project Oberon: The Design of an Operating System and Compiler
|author=Niklaus Wirth, and Jürg Gutknecht
|year=1992
|supp=[http://www.inf.ethz.ch/personal/wirth/ProjectOberon/index.html New, updated edition 2013, with processor in FPGA], the first edition of 1992 is available [http://www.inf.ethz.ch/personal/wirth/ProjectOberon1992.pdf here] with the original [ftp://ftp.ethoberon.ethz.ch/Oberon/Books/ProjectOberon/ code]. 
|description=In 1985 Niklaus Wirth and Juerg Gutknecht embarked on a project to build a new workstation from scratch. The quote from Einstein: 'Make it as simple as possible, but not simpler' served as a signpost for their approach resulting in a system of exemplary lucidity, efficiency and compactness. Wirth was fascinated by the accuracy and reliability of the Voyager space probe then passing Oberon, one of the moons of Uranus. The project was christened Oberon in its honor.

This is the first-hand account of the design, develoment and implementation of [http://www.oberon.ethz.ch/oberon/ Oberon]. It provides a facinating insight into the theory and practice of workstation development for all those eager to learn from the experience of two world class software engineers.

Project [http://www.oberon.ethz.ch/oberon/ Oberon] includes:

* The complete set of program listings for the [http://www.oberon.ethz.ch/oberon/ Oberon] system, together with detailed explanations of how the system was developed
* A demonstration of how economical software engineering can produce a powerful, efficient and flexible system requiring a fraction of the computer power and storage capacity of current commercial operating systems
* An in-depth account of the [http://www.oberon.ethz.ch/oberon/ Oberon] System as the basis for a multi-server workstation, including file distribution, printing and electronic mail facilities.
|review=}}

{{Book|ISBN=0321968972|ASIN=B00O56CFEE
|title=The Design and Implementation of the FreeBSD Operating System, 2nd ed.
|author=Marshall Kirk McKusick, George V. Neville-Neil, and Robert N.M. Watson
|year=2014
|supp=[http://www.informit.com/store/design-and-implementation-of-the-freebsd-operating-9780321968975 Link]
|description=The most complete, authoritative technical guide to the FreeBSD kernel's internal structure has now been extensively updated to cover all major improvements between Versions 5 and 11. Approximately one-third of this edition's content is completely new, and another one-third has been extensively rewritten.

Three long-time FreeBSD project leaders begin with a concise overview of the FreeBSD kernel's current design and implementation. Next, they cover the FreeBSD kernel from the system-call level down–from the interface to the kernel to the hardware. Explaining key design decisions, they detail the concepts, data structures, and algorithms used in implementing each significant system facility, including process management, security, virtual memory, the I/O system, filesystems, socket IPC, and networking.
|review=}}

{| 
|-
| rowspan="6" valign="top" align="center" width="200" |
[[Image:ISBN_0735648735.jpg|120px]]<br/>[http://www.amazon.com/exec/obidos/ASIN/0735648735/osdev-20/ http://wiki.osdev.org/images/3/3a/Buy_from_amazon.gif]<br/><br/>
[[Image:ISBN_0735665877.jpg|120px]]<br/>[http://www.amazon.com/exec/obidos/ASIN/0735665877/osdev-20/ http://wiki.osdev.org/images/3/3a/Buy_from_amazon.gif]
| height="50" | <font size="+1">Windows Internals, 6th ed.</font>
|-
| valign="top" height="10" | '''Author(s):''' Mark Russinovich, David A. Solomon, and Alex Ionescu
|-
| valign="top" height="10" | '''Year:''' 2012
|-
| valign="top" height="10" | '''Supplementary material and/or errata:''' [http://technet.microsoft.com/en-us/sysinternals/bb963901.aspx Link]
|-
| valign="top" | '''Official Description:''' '''''Windows Internals, 6th edition''''' covers the internals of the core kernel components of the Windows 7 and Windows Server 2008 R2 operating systems. This classic book will help you:

* Understand how the core system and management mechanisms work&mdash;from the object manager to services to the registry
* Explore internal system data structures using tools like the kernel debugger
* Grasp the scheduler’s priority and CPU placement algorithms
* Go inside the Windows security model to see how it authorizes access to data
* Understand how Windows manages physical and virtual memory
* Tour the Windows networking stack from top to bottom&mdash;including APIs, protocol drivers, and network adapter drivers
* Troubleshoot file-system access problems and system boot problems
* Learn how to analyze crashes
* Sixth in the series, this edition was again written by Mark Russinovich, a Technical Fellow in Microsoft’s Azure Group, David Solomon, an operating systems expert and Windows internals teacher, and Alex Ionescu, Chief Architect at CrowdStrike and specializing in OS internals and security. Besides updates for changes in Windows, there are many new experiments and examples that highlight the use of both existing and new Sysinternals tools.

The sixth edition is being released in two parts: Part 1 is available now and Part 2 will be available in September. The reason for the split was to get the content into readers’ hands as soon as possible.
|-
| valign="top" | '''Review:'''
|}
<br/>
<br/>

= Programming Languages=

== Theory ==

== Implementations ==

=== C ===

{{Book|ISBN=013089592X|ASIN=013089592X
|title=C: A Reference Manual, 5th ed.
|author=Samuel P. Harbison III, and Guy L. Steele Jr.
|year=2002
|supp=[http://careferencemanual.com/ Link]
|description=This best-selling, authoritative reference manual provides a complete description of the C language, the run-time libraries, and a style of C programming that empha_sizes correctness, portability, and maintainability.

Describing the C language more clearly and in more detail than any other book, authors Samuel P. Harbison and Guy L. Steele Jr. provide in a single manual:

* Standard C (1999) - the new revison of the C Standard supports complex and Boolean types, variable length arrays, precise floating-point programming, and new libraries for portability and internationalization.
* Standard C (1989)- the version of C used by most of today's programmers.
* Traditional C-common practice before 1990, with millions of lines of code in use every day.
* C++ compatible C-code that can be used as C or C++.
* The complete C run-time libraries for all C versions.
* C: A Reference Manual is the only book that describes all the details of C-past and present. It is the single must-have reference for all C programmers and implementors.

Thoroughly revised and updated, the expanded Fifth Edition includes a complete description of the latest C Standard, ISO/IEC 9899:1999, with its powerful language extensions and new libraries.

New! Visit the Web site. www.CAReferenceManual.com contains source code for the longer examples in the book, expanded discussions on language issues, the latest ISO/IEC language corrigenda, and links to other C resources.
|review=}}

{{Book|ISBN=0131103628|ASIN=0131103628
|title=The C Programming Language, 2nd ed.
|author=Brian W. Kernighan, and Dennis M. Ritchie
|year=1988
|supp=[http://cm.bell-labs.com/cm/cs/cbook/ Link]
|description=This book is meant to help the reader learn how to program in C. It is the definitive reference guide, now in a second edition. Although the first edition was written in 1978, it continues to be a worldwide best-seller. This second edition brings the classic original up to date to include the ANSI standard.

From the Preface:

We have tried to retain the brevity of the first edition. C is not a big language, and it is not well served by a big book. We have improved the exposition of critical features, such as pointers, that are central to C programming. We have refined the original examples, and have added new examples in several chapters. For instance, the treatment of complicated declarations is augmented by programs that convert declarations into words and vice versa. As before, all examples have been tested directly from the text, which is in machine-readable form.

As we said in the first preface to the first edition, C "wears well as one's experience with it grows." With a decade more experience, we still feel that way. We hope that this book will help you to learn C and use it well.
|review=}}

{{Book|ISBN=0321563212|ASIN=0321563212
|title=The CERT C Secure Coding Standard
|author=Robert C. Seacord
|year=2008
|supp=[https://www.securecoding.cert.org/confluence/display/seccode/CERT+C+Secure+Coding+Standard Freely available]
|description=Software security has major implications for the operations and assets of organizations, as well as for the welfare of individuals. To create secure software, developers must know where the dangers lie. Secure programming in C can be more difficult than even many experienced  programmers believe.

This book is an essential desktop reference documenting the first official release of '''''The CERT&reg; C Secure Coding Standard'''''. The standard itemizes those coding errors that are the root causes of software vulnerabilities in C and prioritizes them by severity, likelihood of exploitation, and remediation costs. Each guideline provides examples of insecure code as well as secure, alternative implementations. If uniformly applied, these guidelines will eliminate the critical coding errors that lead to buffer overflows, format string vulnerabilities, integer  overflow, and other common software vulnerabilities.
|review=}}

= Security =

{{Book|ISBN=0321304543|ASIN=0321304543
|title=The Art of Computer Virus Research and Defense
|author=Peter Szor
|year=2005
|supp=[http://www.informit.com/store/product.aspx?isbn=0321304543 Link]
|description=Symantec's chief antivirus researcher has written the definitive guide to contemporary virus threats, defense techniques, and analysis tools. Unlike most books on computer viruses, '''''The Art of Computer Virus Research and Defense''''' is a reference written strictly for white hats: IT and security professionals responsible for protecting their organizations against malware. Peter Szor systematically covers everything you need to know, including virus behavior and classification, protection strategies, antivirus and worm-blocking techniques, and much more.

Szor presents the state-of-the-art in both malware and protection, providing the full technical detail that professionals need to handle increasingly complex attacks. Along the way, he provides extensive information on code metamorphism and other emerging techniques, so you can anticipate and prepare for future threats.

Szor also offers the most thorough and practical primer on virus analysis ever published—addressing everything from creating your own personal laboratory to automating the analysis process. This book's coverage includes

* Discovering how malicious code attacks on a variety of platforms
* Classifying malware strategies for infection, in-memory operation, self-protection, payload delivery, exploitation, and more
* Identifying and responding to code obfuscation threats: encrypted, polymorphic, and metamorphic
* Mastering empirical methods for analyzing malicious code—and what to do with what you learn
* Reverse-engineering malicious code with disassemblers, debuggers, emulators, and virtual machines
* Implementing technical defenses: scanning, code emulation, disinfection, inoculation, integrity checking, sandboxing, honeypots, behavior blocking, and much more
* Using worm blocking, host-based intrusion prevention, and network-level defense strategies
|review=}}

{{Book|ISBN=0764574817|ASIN=0764574817
|title=Reversing: Secrets of Reverse Engineering
|author=Eldad Eilam
|year=2005
|supp=[http://eu.wiley.com/WileyCDA/WileyTitle/productCd-0764574817.html Link]
|description=Beginning with a basic primer on reverse engineering-including computer internals, operating systems, and assembly language-and then discussing the various
applications of reverse engineering, this book provides readers with practical, in-depth techniques for software reverse engineering. The book is broken into two parts, the first deals with security-related reverse engineering and the second explores the more practical aspects of reverse engineering. In addition, the author explains how to reverse engineer a third-party software library to improve interfacing and how to reverse engineer a competitor's software to build a better product.
* The first popular book to show how software reverse engineering can help defend against security threats, speed up development, and unlock the secrets of competitive products
* Helps developers plug security holes by demonstrating how hackers exploit reverse engineering techniques to crack copy-protection schemes and identify software targets for viruses and other malware
* Offers a primer on advanced reverse-engineering, delving into "disassembly"-code-level reverse engineering-and explaining how to decipher assembly language
|review=}}

{{Book|ISBN=0470068523|ASIN=0470068523
|title=Security Engineering, 2nd ed.
|author=Ross Anderson
|year=2008
|supp=[http://www.cl.cam.ac.uk/~rja14/book.html Link]
|description=The world has changed radically since the first edition of this book was published in 2001. Spammers, virus writers, phishermen, money launderers, and spies now trade busily with each other in a lively online criminal economy and as they specialize, they get better. In this indispensable, fully updated guide, Ross Anderson reveals how to build systems that stay dependable whether faced with error or malice. Here s straight talk on critical topics such as technical engineering basics, types of attack, specialized protection mechanisms, security psychology, policy, and more.
|review=}}

== Cryptology ==

=== Cryptanalysis ===

=== Cryptography ===

{{Book|ISBN=0471117099|ASIN=0471117099
|title=Applied Cryptography, 2nd ed.
|author=Bruce Schneier
|year=1996
|supp=[http://www.schneier.com/book-applied.html Link]
|description=This new edition of the cryptography classic provides you with a comprehensive survey of modern cryptography. The book details how programmers and electronic communications professionals can use cryptography -- the technique of enciphering and deciphering messages -- to maintain the privacy of computer data. It describes dozens of cryptography algorithms, gives practical advice on how to implement them in cryptographic software, and shows how they can be used to solve security problems. Covering the latest developments in practical cryptographic techniques, this new edition shows programmers who design computer applications, networks, and storage systems how they can build security into their software and systems.

'''What's New in the Second Edition?'''

The second edition of Applied Cryptography is a major rewrite of the first edition: 50% more words, 7 more chapters, and over 1600 new references. Not only did I make corrections to the first edition and add developments since it was published, but I also included topics left out of the first edition.
The second edition has lots of new algorithms (including GOST, Blowfish, RC4, and A5), more information on the Clipper Chip and key escrow, dozens of new protocols, more information on how PGP works, detailed information on key management and modes of operation, and new source code.
|review=}}

= Software Development =

== Theory ==

{{Book|ISBN=0735619670|ASIN=0735619670
|title=Code Complete, 2nd ed.
|author=Steve McConnell
|year=2004
|supp=[http://www.cc2e.com/ Link]
|description=For more than a decade, Steve McConnell, one of the premier authors and voices in the software community, has helped change the way developers write code--and produce better software. Now his classic book, CODE COMPLETE, has been fully updated and revised with best practices in the art and science of constructing software. Whether you're a new developer seeking a sound introduction to the practice of software development or a veteran exploring strategic new approaches to problem solving, you'll find a wealth of practical suggestions and methods for strengthening your skills. Topics include design, applying good techniques to construction, eliminating errors, planning, managing construction activities, and relating personal character to superior software. This new edition features fully updated information on programming techniques, including the emergence of Web-style programming, and integrated coverage of object-oriented design. You'll also find new code examples--both good and bad--in C++, Microsoft(r) Visual Basic(r), C#, and Java, though the focus is squarely on techniques and practices.
|review=}}

{{Book|ISBN=0201835959|ASIN=0201835959
|title=The Mythical Man-Month: Essays on Software Engineering, 20th Anniversary Edition
|author=Frederick P. Brooks, Jr.
|year=1995
|supp=N/A
|description=Few books on software project management have been as influential and timeless as '''The Mythical Man-Month'''. With a blend of software engineering facts and thought-provoking opinions, Fred Brooks offers insight for anyone managing complex projects. These essays draw from his experience as project manager for the IBM System/360 computer family and then for OS/360, its massive software system. Now, 20 years after the initial publication of his book, Brooks has revisited his original ideas and added new thoughts and advice, both for readers already familiar with his work and for readers discovering it for the first time.
 
The added chapters contain (1) a crisp condensation of all the propositions asserted in the original book, including Brooks' central argument in '''The Mythical Man-Month:''' that large programming projects suffer management problems different from small ones due to the division of labor; that the conceptual integrity of the product is therefore critical; and that it is difficult but possible to achieve this unity; (2) Brooks' view of these propositions a generation later; (3) a reprint of his classic 1986 paper "No Silver Bullet"; and (4) today's thoughts on the 1986 assertion, "There will be no silver bullet within ten years."
|review=}}

== Practice ==

{{Book|ISBN=0201633612|ASIN=0201633612
|title=Design Patterns: Elements of Reusable Object-Oriented Software
|author=Erich Gamma, Richard Helm, Ralph Johnson, and John Vlissides
|year=1994
|supp=
|description=Capturing a wealth of experience about the design of object-oriented software, four top-notch designers present a catalog of simple and succinct solutions to commonly occurring design problems. Previously undocumented, these 23 patterns allow designers to create more flexible, elegant, and ultimately reusable designs without having to rediscover the design solutions themselves.

The authors begin by describing what patterns are and how they can help you design object-oriented software. They then go on to systematically name, explain, evaluate, and catalog recurring designs in object-oriented systems. With Design Patterns as your guide, you will learn how these important patterns fit into the software development process, and how you can leverage them to solve your own design problems most efficiently.

Each pattern describes the circumstances in which it is applicable, when it can be applied in view of other design constraints, and the consequences and trade-offs of using the pattern within a larger design. All patterns are compiled from real systems and are based on real-world examples. Each pattern also includes code that demonstrates how it may be implemented in object-oriented programming languages like C++ or Smalltalk.
|review=}}

{{Book|ISBN=0596009658|ASIN=0596009658
|title=Learning the bash Shell, 3rd ed.
|author=Cameron Newham, and Bill Rosenblatt
|year=2005
|supp=[http://shop.oreilly.com/product/9780596009656.do Free book, errata, and code], [http://oreilly.com/catalog/bash3/errata/ more errata]
|description=O'Reilly's bestselling book on Linux's bash shell is at it again. Now that Linux is an established player both as a server and on the desktop ''Learning the bash Shell'' has been updated and refreshed to account for all the latest changes. Indeed, this third edition serves as the most valuable guide yet to the bash shell.

As any good programmer knows, the first thing users of the Linux operating system come face to face with is the shell the UNIX term for a user interface to the system. In other words, it's what lets you communicate with the computer via the keyboard and display. Mastering the bash shell might sound fairly simple but it isn't. In truth, there are many complexities that need careful explanation, which is just what ''Learning the bash Shell'' provides.

If you are new to shell programming, the book provides an excellent introduction, covering everything from the most basic to the most advanced features. And if you've been writing shell scripts for years, it offers a great way to find out what the new shell offers. ''Learning the bash Shell'' is also full of practical examples of shell commands and programs that will make everyday use of Linux that much easier. With this book, programmers will learn:

* How to install bash as your login shell
* The basics of interactive shell use, including UNIX file and directory structures, standard I/O, and background jobs
* Command line editing, history substitution, and key bindings

* How to customize your shell environment without programming

* The nuts and bolts of basic shell programming, flow control structures, command-line options and typed variables

* Process handling, from job control to processes, coroutines and subshells

* Debugging techniques, such as trace and verbose modes

* Techniques for implementing system-wide shell customization and features related to system security
|review=}}

{{Book|ISBN=059652983X|ASIN=059652983X
|title=Learning the vi and Vim Editors, 7th ed.
|author=Arnold Robbins, Elbert Hannah, and Linda Lamb
|year=2008
|supp=[http://shop.oreilly.com/product/9780596529833.do Link]
|description=There's nothing that hard-core Unix and Linux users are more fanatical about than their text editor. Editors are the subject of adoration and worship, or of scorn and ridicule, depending upon whether the topic of discussion is your editor or someone else's.

''vi'' has been the standard editor for close to 30 years. Popular on Unix and Linux, it has a growing following on Windows systems, too. Most experienced system administrators cite vi as their tool of choice. And since 1986, this book has been the guide for ''vi''.

However, Unix systems are not what they were 30 years ago, and neither is this book. While retaining all the valuable features of previous editions, the 7th edition of ''Learning the vi and vim Editors'' has been expanded to include detailed information on ''vim'', the leading ''vi'' clone. ''vim'' is the default version of vi on most Linux systems and on Mac OS X, and is available for many other operating systems too.

With this guide, you learn text editing basics and advanced tools for both editors, such as multi-window editing, how to write both interactive macros and scripts to extend the editor, and power tools for programmers -- all in the easy-to-follow style that has made this book a classic.

''Learning the vi and vim Editors'' includes:

'''A complete introduction to text editing with vi:'''

* How to move around ''vi'' in a hurry
* Beyond the basics, such as using buffers
* ''vi'''s global search and replacement
* Advanced editing, including customizing ''vi'' and executing Unix commands

'''How to make full use of vim:'''
* Extended text objects and more powerful regular expressions
* Multi-window editing and powerful ''vim'' scripts
* How to make full use of the GUI version of ''vim'', called ''gvim''
* ''vim'''s enhancements for programmers, such as syntax highlighting, folding and extended tags

Coverage of three other popular ''vi'' clones -- ''nvi'', ''elvis'', and ''vile'' -- is also included. You'll find several valuable appendixes, including an alphabetical quick reference to both ''vi'' and ''ex'' mode commands for regular ''vi'' and for ''vim'', plus an updated appendix on ''vi'' and the Internet.

Learning either ''vi'' or ''vim'' is required knowledge if you use Linux or Unix, and in either case, reading this book is essential. After reading this book, the choice of editor will be obvious for you too.
|review=}}

{{Book|ISBN=0596006101|ASIN=0596006101
|title=Managing Projects with GNU Make, 3rd ed.
|author=Robert Mecklenburg
|year=2004
|supp=[http://shop.oreilly.com/product/9780596006105.do Free book, errata, and code], [http://oreilly.com/catalog/make3/errata/ more errata]
|description=The utility simply known as ''make'' is one of the most enduring features of both Unix and other operating systems. First invented in the 1970s, ''make'' still turns up to this day as the central engine in most programming projects; it even builds the Linux kernel. In the third edition of the classic ''Managing Projects with GNU make'', readers will learn why this utility continues to hold its top position in project build software, despite many younger competitors.

The premise behind ''make'' is simple: after you change source files and want to rebuild your program or other output files, make checks timestamps to see what has changed and rebuilds just what you need, without wasting time rebuilding other files. But on top of this simple principle, ''make'' layers a rich collection of options that lets you manipulate multiple directories, build different versions of programs for different platforms, and customize your builds in other ways.

This edition focuses on the GNU version of ''make'', which has deservedly become the industry standard. GNU make contains powerful extensions that are explored in this book. It is also popular because it is free software and provides a version for almost every platform, including a version for Microsoft Windows as part of the free Cygwin project. ''Managing Projects with GNU make'', 3rd Edition provides guidelines on meeting the needs of large, modern projects. Also added are a number of interesting advanced topics such as portability, parallelism, and use with Java.

Robert Mecklenburg, author of the third edition, has used ''make'' for decades with a variety of platforms and languages. In this book he zealously lays forth how to get your builds to be as efficient as possible, reduce maintenance, avoid errors, and thoroughly understand what make is doing. Chapters on C++ and Java provide makefile entries optimized for projects in those languages. The author even includes a discussion of the makefile used to build the book.
|review=}}

{{Book|ISBN=0138297711|ASIN=0138297711
|title=Source File Management with SCCS
|author=Israel Silverberg
|year=1991
|supp=N/A
|description=Oriented towards the needs of senior software engineers, software managers, system administrators, and configuration managers, this volume focuses on the building and maintenance of a source file library in the UNIX System V environment. The book gives an explanation of the concept of the UNIX source code control system, describes in detail the utilities that constitute SCCS, and shows how to use these tools to build and maintain a source file library. It also shows how to place all applications in a single library and how that library can be used by different departments. Techniques and additional tools for maintaining source files at the product level are explained with a view to allowing readers to establish a source file management system that will trace versions of an entire application instead of a single source file. The book then explains how to establish a secure source library where access to read and/or change is strictly controlled. Finally, the book provides source code for the special utilities needed to implement the concepts described. A disk of source codes is available separately.
|review=}}

= Theory of Computation =

{{Book|ISBN=113318779X|ASIN=113318779X
|title=Introduction to the Theory of Computation, 3rd ed.
|author=Michael Sipser
|year=2012
|supp=[http://www.cengage.com/search/productOverview.do?Ntt=introduction+to+the+theory+of+computation&x7c;&x7c;991938321204392568016032450701059771599&N=4294922451&Ntk=all%7C%7CP_EPI%7C%7CP_EPI&x7c%3B&x7c%3B991938321204392568016032450701059771599 Link], [http://math.mit.edu/~sipser/book.html Errata]
|description=Now you can clearly present even the most complex computational theory topics to your students with Sipser's distinct, market-leading INTRODUCTION TO THE THEORY OF COMPUTATION, 3E. The number one choice for today's computational theory course, this highly anticipated revision retains the unmatched clarity and thorough coverage that make it a leading text for upper-level undergraduate and introductory graduate students. This edition continues author Michael Sipser's well-known, approachable style with timely revisions, additional exercises, and more memorable examples in key areas. A new first-of-its-kind theoretical treatment of deterministic context-free languages is ideal for a better understanding of parsing and LR(k) grammars. This edition's refined presentation ensures a trusted accuracy and clarity that make the challenging study of computational theory accessible and intuitive to students while maintaining the subject's rigor and formalism. Readers gain a solid understanding of the fundamental mathematical properties of computer hardware, software, and applications with a blend of practical and philosophical coverage and mathematical treatments, including advanced theorems and proofs. INTRODUCTION TO THE THEORY OF COMPUTATION, 3E's comprehensive coverage makes this an ideal ongoing reference tool for those studying theoretical computing.
|review=}}

{| 
|-
| rowspan="6" valign="top" align="center" width="200" |
[[Image:ISBN_0716782669.jpg|120px]]<br/>[http://www.amazon.com/exec/obidos/ASIN/0716782669/osdev-20/ http://wiki.osdev.org/images/3/3a/Buy_from_amazon.gif]<br/><br/>
[[Image:ISBN_071678291X.jpg|120px]]<br/>[http://www.amazon.com/exec/obidos/ASIN/071678291X/osdev-20/ http://wiki.osdev.org/images/3/3a/Buy_from_amazon.gif]
| height="50" | <font size="+1">The Language of Machines: An Introduction to Computability and Formal Languages</font>
|-
| valign="top" height="10" | '''Author(s):''' Robert W Floyd, and Richard Beigel
|-
| valign="top" height="10" | '''Year:''' 1994
|-
| valign="top" height="10" | '''Supplementary material and/or errata:''' [http://web.archive.org/web/20100117200711/http://knight.cis.temple.edu/~beigel/long.html#lom Link]
|-
| valign="top" | '''Official Description:''' In ''The Language of Machines'', Robert Floyd and Richard Beigel revolutionize the teaching of computability and languages. They propose nothing less than redefinition of the building blocks of automata theory: their unified model of computation clarifies the subject as never before. Floyd and Beigel's single model encompasses all the traditional types of computing machines and even "real world" electronic computers.

Using an approach that has been successfully class tested at Stanford, Yale, and Johns Hopkins, Floyd and Beigel offer valuable innovations:

* '''Unified definitions''' that yield insight into the capabilities of a wide variety of machines
* '''The perfect combination of simplicity and rigor'''---their new approach makes formerly obscure results accessible.
* '''Ideas and examples from practical computer science''' bring theory to life
* '''A mechanism to combine programs,''' akin to pipes in UNIX(tm)
* '''The first formal definition of simulation''' that permits modularized proofs
* '''A general system of standardization of programs,''' which streamlines proofs

Floyd an Beigel's bold reformulation of computability and formal language theory provides a firm foundation on which students can build a rich and enduring body of knowledge.
|-
| valign="top" | '''Review:''' 
|}
<br/>
<br/>

= UNIX =

{{Book|ISBN=0321637739|ASIN=0321637739
|title=Advanced Programming in the UNIX Environment, 3rd ed.
|author=Stephen A. Rago, and W. Richard Stevens
|year=2013
|supp=[http://www.apuebook.com/ Link]
|description=Master the system calls at the heart of today's UNIX and Linux systems: the programming interfaces that drive the UNIX and Linux kernels and run everything from filesystems and multitasking to interprocess communication. A thorough knowledge of system calls is essential for every UNIX and Linux programmer -- and for 20 years, W. Richard Stevens' ''Advanced Programming in the Unix Environment'' has delivered that. Now, Stephen Rago has completely updated this classic to reflect modern distributions including Red Hat 2.6.x, Solaris 10 3, OS X 10.5.4, FreeBSD 7.0.5 -- and for the first time, Ubuntu. As in previous editions, Rago begins with essentials such as file and process control, carefully laying the groundwork for more advanced techniques. He supports his crystal-clear explanations with nearly 10,000 lines of code, all carefully tested on each of these leading versions and distributions. Rago begins with an up-to-date overview of the UNIX System, its standardization processes, and its diverse implementations. Next, he introduces file I/O, files, directories, and the Standard I/O Library. You'll walk through working with UNIX/Linux system data files and information… controlling the process environment and its relationships… using signals, threads, and daemon processes… mastering advanced I/O techniques, interprocess communication, sockets, and advanced IPC… using Terminal I/O and Pseudo Terminals… working with database libraries, communicating with network printers, and much more. Appendices provide essential information on function prototypes, miscellaneous source code, and solutions to many of this guide's exercises. ''For all intermediate-to-advanced level UNIX programmers and software engineers.''
|review=}}

{{Book|ISBN=0139319573|ASIN=0139319573
|title=UNIX Curses Explained
|author=Berny Goodheart
|year=1991
|supp=N/A
|description=Fully documents Curses library and provides a detailed explanation of UNIX Curses. Contains a full alphabetical reference section and many clear examples using Curses, Windows, color manipulation, alternative character sets, pads, and terminals.
|review=}}

{| 
|-
| rowspan="6" valign="top" align="center" width="200" |
[[Image:ISBN_0131411551.jpg|120px]]<br/>[http://www.amazon.com/exec/obidos/ASIN/0131411551/osdev-20/ http://wiki.osdev.org/images/3/3a/Buy_from_amazon.gif]<br/><br/>
[[Image:ISBN_0130810819.jpg|120px]]<br/>[http://www.amazon.com/exec/obidos/ASIN/0130810819/osdev-20/ http://wiki.osdev.org/images/3/3a/Buy_from_amazon.gif]
| height="50" | <font size="+1">UNIX Network Programming</font>
|-
| valign="top" height="10" | '''Author(s):''' Andrew M. Rudoff, Bill Fenner, and W. Richard Stevens
|-
| valign="top" height="10" | '''Year:''' 2003 (Vol1), 1998 (Vol2)
|-
| valign="top" height="10" | '''Supplementary material and/or errata:''' [http://www.unpbook.com/ Volume 1], [http://www.kohala.com/start/unpv22e/unpv22e.html Volume 2]
|-
| valign="top" | '''Official Description:''' This book is for people who want to write programs that communicate with each other using an application program interface (API) known as sockets. Some readers may be very familiar with sockets already, as that model has become synonymous with network programming. Others may need an introduction to sockets from the ground up. The goal of this book is to offer guidance on network programming for beginners as well as professionals, for those developing new network-aware applications as well as those maintaining existing code, and for people who simply want to understand how the networking components of their system function.

All the examples in this text are actual, runnable code tested on Unix systems. However, many non-Unix systems support the sockets API and the examples are largely operating system-independent, as are the general concepts we present. Virtually every operating system (OS) provides numerous network-aware applications such as Web browsers, email clients, and file-sharing servers. We discuss the usual partitioning of these applications into client and server and write our own small examples of these many times throughout the text.

Well-implemented interprocess communications (IPC) are key to the performance of virtually every non-trivial UNIX program. In ''UNIX Network Programming, Volume 2,'' Second Edition, legendary UNIX expert W. Richard Stevens presents a comprehensive guide to every form of IPC, including message passing, synchronization, shared memory, and Remote Procedure Calls (RPC).

Stevens begins with a basic introduction to IPC and the problems it is intended to solve. Step-by-step you'll learn how to maximize both System V IPC and the new Posix standards, which offer dramatic improvements in convenience and performance. You'll find extensive coverage of Pthreads, with many examples reflecting multiple threads instead of multiple processes. Along the way, you'll master every current IPC technique and technology, including:

* Pipes and FIFOs.
* Posix and System V Message Queues
* Mutexes and Condition Variables
* Read-Write Locks
* Record Locking
* Posix and System V Semaphores
* Posix and System V Shared Memory
* Solaris Doors and Sun RPC
* Performance Measurements of IPC Techniques

If you've read Stevens' best-selling first edition of UNIX Network Programming, this book expands its IPC coverage by a factor of five! You won't just learn about IPC "from the outside." You'll actually create implementations of Posix message queues, read-write locks, and semaphores, gaining an in-depth understanding of these capabilities you simply can't get anywhere else.

The book contains extensive new source code-all carefully optimized and available on the Web. You'll even find a complete guide to measuring IPC performance with message passing bandwidth and latency programs, and thread and process synchronization programs.

The better you understand IPC, the better your UNIX software will run. One book contains all you need to know: UNIX Network Programming, Volume 2, Second Edition.
|-
| valign="top" | '''Review:''' 
|}
<br/>
<br/>

{{Book|ISBN=013937681X|ASIN=013937681X
|title=The Unix Programming Environment
|author=Brian W. Kernighan, and Rob Pike
|year=1983
|supp=[http://cm.bell-labs.com/cm/cs/upe/index.html Link]
|description=Designed for first-time and experienced users, this book describes the UNIX&reg; programming environment and philosophy in detail. Readers will gain an understanding not only of how to use the system, its components, and the programs, but also how these fit into the total environment.
|review=}}

{{Book|ISBN=1695978552|ASIN=1695978552
|title=UNIX: A History and a Memoir
|author=Brian W. Kernighan
|year=2019
|supp=
|description=The fascinating story of how Unix began and how it took over the world. Brian Kernighan was a member of the original group of Unix developers, the creator of several fundamental Unix programs, and the co-author of classic books like "The C Programming Language" and "The Unix Programming Environment."
|review=}}

= See also =

* [http://forum.osdev.org/viewtopic.php?f=1&t=6989&p=174472 Book discussions on the forums]

[[Category:Resources]]
````

<a id="w001573"></a>

## W001573 — Category:Languages

Bron-ID: W001573
Namespace: Category (14)
Revisie: 8458 | Datum: 2009-07-09T22:07:11Z
Bron: https://wiki.osdev.org/Category:Languages
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ALanguages&oldid=8458

````text
There are many [[languages|programming languages]], some more suited for OS development and kernel writing than others.

[[Category:Main Page Topics]]
````

<a id="w001574"></a>

## W001574 — Category:Book Covers

Bron-ID: W001574
Namespace: Category (14)
Revisie: 2339 | Datum: 2006-12-16T20:57:48Z
Bron: https://wiki.osdev.org/Category:Book_Covers
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ABook_Covers&oldid=2339

````text
Books covers related to operating system development. These are used on the [[Books]] page. Image sizes should be less than 200 pixels in width and the uploaded files name should be ISBN_#.jpg where # is the ISBN number. The format should be always jpg.

[[Category:Resources]]
````

<a id="w001576"></a>

## W001576 — Talk:Text UI

Bron-ID: W001576
Namespace: Talk (1)
Revisie: 14055 | Datum: 2012-10-23T09:58:12Z
Bron: https://wiki.osdev.org/Talk:Text_UI
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AText_UI&oldid=14055

````text
==Text Mode 0==
Confusing term - especially when talking about 80x25 mode - would "VGA mode 3" be better? - [[User:Combuster|Combuster]]

Mode 0 refers to the screen mode the BIOS sets upon loading the bootloader. It's the most commonly used screen mode for text user interfaces. 80x25 refers to the number of characters on the screen - 80 characters per line and 25 lines per screen, not the number of pixels. On a VGA adapter, the actual screen resolution is 720x400 pixels with 16 colours avaliable, and each character is 9x8 pixels. - [[User:MessiahAndrw|MessiahAndrw]]

I know exactly what you are referring to, the question was however wether to use 'Mode 0' or not - I think its confusing for those with few or no VGA programming experience.. ''As a side note, plz dont append at the top.'' - [[User:Combuster|Combuster]] 05:46, 15 December 2006 (CST)


==Move?==
IMHO, this article should be moved into the "Video" category of the "Hardware" section. As it deals much more with the hardware aspect of printing characters to screen than to the "theory" behind it. -[[User:Jhawthorn|Jhawthorn]] 03:28, 15 December 2006 (CST)

I have the idea this article comes from the 'How to print text' faq page. I suggest splitting it into a theory and a vga text mode page, with appropriate categories for each - [[User:Combuster|Combuster]] 05:46, 15 December 2006 (CST)

"..." No, I wrote it myself.
--[[User:MessiahAndrw|MessiahAndrw]] 05:38, 24 May 2007 (CDT)

The problem didn't go away, which is what made me check the discussion page. Another problem is that of terminology. When people speak of TUI's, they think of GUI's rendered on an alphanumeric video mode; when they speak of CLI's, they mean shells. This is also consistent with Wikipedia. --[[User:Love4boobies|Love4boobies]] 04:58, 23 October 2012 (CDT)
````

<a id="w001577"></a>

## W001577 — What Order Should I Make Things In?

Bron-ID: W001577
Namespace: Main (0)
Revisie: 29273 | Datum: 2024-10-09T17:06:17Z
Bron: https://wiki.osdev.org/What_Order_Should_I_Make_Things_In%3F
Revisie-URL: https://wiki.osdev.org/index.php?title=What_Order_Should_I_Make_Things_In%3F&oldid=29273

````text
This is a question of style. You can start at the start and dig straight in, writing a bootsector, then a minimal kernel, and build from there. You could skip the bootsector, and use a ready-made bootloader like [[GRUB]] (it's open for discussion whether [[Rolling Your Own Bootloader|rolling your own bootloader]] is a valuable experience or a waste of time). You can also write bits and pieces in no specific order, and just put them together at the very end. There is probably no right or wrong way to go about doing it. If you would like an overview of things that are handy to get working (and are practically a must for every OS), you can keep on reading.

== For the starter ==

# Printing strings and integer numbers (both decimal and hex) on the screen is certainly a must. This is one of most basic ways of debugging, and virtually all of us have gone through a kprint() or kout in version 0.1.0.
# Outputting to a serial port will save you a lot of debugging time. You don't have to fear losing information due to scrolling. You will be able to test your OS from a console, filter interesting debug messages, and automatize some tests.
# Having a working and reliable interrupt/exception handling system that can dump the contents of the registers (and perhaps the address of the fault) will be very useful. 
# Plan your memory map (virtual, and physical) : decide where you want the data to be
# The heap: allocating memory at runtime ('''malloc''' and '''free''') is almost impossible to go without. It should be implemented as soon as possible.

Once those steps are completed, whether you'll try to have a working GUI before you have a filesystem, multitasking or module-loading is completely up to you. Try to sketch out what is likely to depend on what, and do things in 'least dependent first' order.

For instance, the GUI could depend on the filesystem to load bitmaps or resources, but you don't necessarily need bitmaps in your very first GUI. Good advice in such a case is to design the interface of the filesystem first (be it open/close/read/write or something else), and then go on with whatever you prefer, respecting the interface on both sides.

== Style Extremes ==
OSDev seems to have "archetypes" among the OSDevers (and the OS they develop). Of course, the idea that most people have of a "complete" kernel includes most (if not all) of the items listed below.

=== Lino Commando ===
{{main|Lino Commando}}
He's been impressed by the "naked beauty" of DOS times. The first thing he wants to have is a blinking cursor after a ">" symbol so that he can type commands. No matter if there's no way to start two programs at the same time: he just needs a text editor and a filesystem driver.

=== Nick Stacky ===
{{main|Nick Stacky}}
His test-machine has no keyboard and no screen (no one needs that). All he needs is NICs (Network Interface Cards)... plenty of NICs... and an Ethernet cable to see if his kernel responds to pings and routes packets correctly. His kernel has powerful multithreading and a complete TCP/IP stack, of course.

=== James T. Klik ===
{{main|James T. Klik}}
Look through the window: see that background with alpha-blending and the anti-aliased fonts? See the corner of the screen? That's Klik's start menu! Hmm, no: I have no so-called 'programs' folder, as I can't load any applications for now, but here's a 16-items list of test cases for my WidgetToolKit.

=== Eleanore Semaphore ===
{{main|Eleanore Semaphore}}
Works in a dark corner of a small room with listings all around. Her system has much evolved since last year, though all she can show you is still the same: bunches of A's and B's displayed in no obvious order on a text console that she controls with magic key combinations. You hear her talking about how she implemented component programming, auto-dependency-resolving and a virtual clock algorithm for her scheduler.

=== Alta Lang ===
{{main|Alta Lang}}
Why would anyone want to write something as complex as an operating system in something as clunky and old as C? Alta wants something different for her system: a new language for a more elegant operating system. She spends at least as much time on language design as on system design, in the hopes that if she can design the language to be clean enough and powerful enough, the system will simply fall into place.

=== Stan Dard ===
{{main|Stan Dard}}
Stan knows how things should be. It's all nicely written down in standards. He sees how they fit together to form a beautiful system, well, if you disregard all the parts where the standard is stupid. He has a reasonably complete implementation of the areas he knows about and not-terribly-impressive code for the areas he is unskilled in. 

=== Richard Theseus ===
{{main|Richard Theseus}}
Richard likes his current OS very much, except for one thing: it's not written by him. Bit by bit, he wants to replace the parts of his current OS with his own code. His ultimate aim is to have an identical copy of his host OS, but with his code.

=== Mister Perfect ===
{{main|Mister Perfect}}
Mister Perfect wants to write a perfect operating system, as he looks at all the flaws current operating systems have. He researches many design and implementation possibilities in order to decide what is better. He starts over multiple times in order to assure quality and perfectness.

=== Andy Microbaum ===
{{main|Andy Microbaum}}
Andy is a fan of small kernels who put much stuff done in a monolithic kernel into userspace.

=== Dizz Tracked ===
{{main|Dizz Tracked}}
Dizz gets distracted by unneccessary side projects.

=== Mono Lizzy ===
{{main|Mono Lizzy}}
Lizzy likes to put everything into one big kernel.

=== Ideas for new Archetypes ===

* '''Edward Scissorhands''' - Sounds like something designed on paper to do everything but completely broken in practice, I see architecture astronauts rather doing vaporware (e.g. you'd never get to see the Scissorhanded implementation the architecture would require)
* '''Real Man''' - Writes an entire operating system in Assembly, preferably in real mode. Abhors [http://en.wikipedia.org/wiki/Real_Men_Don't_Eat_Quiche Quiche Eaters] and [http://en.wikipedia.org/wiki/Real_Programmers_Don%27t_Use_Pascal Pascal].
* '''Vizier Studio''' - Visual Studio people.
* '''Barry Pi''' - ARM people.
* '''Racing Carl''' - Does all in protected mode Assembler and rewrites the latest 3D library and tries to squeeze the last bit of speed out of the code.

=== Archetypes to avoid ===

* '''[[Cowboy Coder]]''' - Someone who eagerly jumps into coding without any plan.
* '''[[Dae Dreamer]]'''- Someone who spends all their time day dreaming and reading about operating systems, but hasn't yet started one.
* '''[[Duct von Tape]]''' - Someone that reuses a lot of code and combines it into a horrible frankenstein of a project. Or perhaps [[:Category:Lovecraftian|lovecraftian]]?
* '''[[Sir Dunning-Kruger]]''' - Someone overestimating his/her skills dramatically.

[[Category:OS Development]]
[[Category:FAQ]]
````

<a id="w001578"></a>

## W001578 — Eleanore Semaphore

Bron-ID: W001578
Namespace: Main (0)
Revisie: 24892 | Datum: 2020-06-22T13:53:15Z
Bron: https://wiki.osdev.org/Eleanore_Semaphore
Revisie-URL: https://wiki.osdev.org/index.php?title=Eleanore_Semaphore&oldid=24892

````text
Eleanore Semaphore works in a darky corner of a small room with listings all around. Her system has evolved greatly since last year, though all she can show you is still the same: bunches of A and B displayed in no-obvious-order on a text console and that she controls with magic key combos. Hear her talking about how she implemented components programming, auto-dependencies resolving and virtual clock algorithm for her scheduler. Maybe you prefer a [[What order should I make things in|different style]].

== The foundation of a kernel ==

Ok, so you know what a [[Semaphore|semaphore]] is? Or maybe [[Monitor|monitors]] or just plain [[Message Passing|message passing]] would be a better way to synchronize? 'Cos I guess we want to synchronize our processes. You want to write a [[Microkernel|µ-kernel]] don't you, even if you actually wanted to run some tasks in the kernel for performance later?

Honestly, if you are like Eleanore, you probably have a pretty good idea of what you want the "big picture" to feel like, but little idea of how to get there. In fact, you'll probably be reading research papers pretty soon, so maybe the real answer hasn't been invented yet? And whether you are going to write just a barebones microkernel with the most simple possible [[Message_Passing|IPC]] system, or a sophisticated distributed [[RPC]] model with [[Garbage Collection|garbage collected]] components, you are going to need some basics first.

And we are going to dive straight into the theory, but I guess that won't scare you, right?

=== Threads, and maybe processes ===

We are going to be multi-threading, right? So we need [[Thread|threads]]. Personally I'd start from the basic threads, because lots of ofter stuff depends on them. You don't need to actually do any multi-threading in your kernel, but that might be the easiest thing. In any case, you probably want some kind of [[Scheduler|scheduler]] in your kernel. At least enough that you can tell a blocked thread from a ready-to-run thread.

Whether you really want processes at this point depends. The simple thing to do is to make them just some dummy object at this point, and flesh them out when [[Memory Management|virtual memory management]] is implemented.

=== Synchronization ===

You will then need some kind of syncronization primitives; probably at least some kind of semaphores. And while you can definitely get [[Mutual Exclusion|mutual exclusion]] with nothing but [[Spinlock|spinlocks]], you need a scheduler so that you can make your threads blocked, so you don't need to busy-loop.

And once we have synchronization in place, we are ready to start printing As and Bs in a predictable order, right?

=== Userspace ===

Since a µ-kernel is nothing without userspace processes, you probably want to go into userspace relatively early. You could simulate them in kernel of course, and you can load them using GRUB or whatever you load your kernel with. But for real userspace you need two things. And you could do them in any order really.

And really, if you are like me, then you probably agree that these two are what makes an operating system different.

==== Virtual memory management ====

You probably need a manager for virtual memory and this is a good moment to design one. You have several options, like always:

* write a normal VMM in kernel
* write only the minimum in kernel, and support either:
** separate pagers in userspace, or
** self-paging

If you are going to support some kinds of userspace management (and even if you don't), you should probably consider how your IPC and VMM will interact before you start writing any high-level management code. But then again, most existing systems seem to be quite happy with a "regular" VMM.

Existing virtual memory management schemes that you might find useful include those in Unix, including [http://www.google.com/search?ie=utf-8&oe=utf-8&q=linux%20memory%20management Linux], FreeBSD and others, VMS, [http://msdn.microsoft.com/library/en-us/dngenlib/html/msdn_ntvmm.asp Windows NT] (and [http://www.winntmag.com/Windows/Articles/ArticleID/4494/pg/1/1.html VMS and NT similarities]), [http://citeseer.ist.psu.edu/rashid87machineindependent.html Mach] (and [http://developer.apple.com/documentation/Darwin/Conceptual/KernelProgramming/Mach/chapter_6_section_5.html Darwin]), [http://os.inf.tu-dresden.de/L4/doc.html L4], [http://www.cl.cam.ac.uk/Research/SRG/netos/old-projects/pegasus/publications/osdi99/paper.html Nemesis] ([http://www.usenix.org/publications/library/proceedings/osdi99/full_papers/hand/hand.pdf PDF]).

ToDo: should this stuff go to a separate "Virtual Memory" page?

==== How can my processes talk to each other? ====

For any future processes to be able to talk to each other (and eventually outside world) one needs some facilities for communication. What kind of [[Category:IPC|inter-process communication]] you are going to have, depends on what exactly do you want your system to feel like, but you definitely want something. Some of the alternatives available are

* Unix style: [[Unix Pipes|pipes]], [[Socket|sockets]], [[File|files]], [[select]]
* [[Message Passing]], or maybe [[RPC|Remote Procedure Calls]]
* [[Shared Memory]]
* maybe something totally new?

There are almost endless number of different possibilities and several variations of each.

== Eleanore's Bookshelf ==
Well, most of these aren't books, actually. More likely the proceedings of USENIX or Hot Topics in Operating Systems, stacks of printed articles from various universities ... She also has most of Andy's writings and will brew him a cup of fresh coffee when Linus comes back for the debates.

== Eleanore Semaphore's opponents position ==

A sophisticated IPC model and component objects are great, but what good are they if one can't communicate with the outside world? And what good is a sophisticated scheduler if all it can do is affect the order some As and Bs are printed on screen? And how can distributed systems be useful, if there is no working network? It's good to research things, but do you guys ever produce anything practical?

== People and OSes related to Eleanore Semaphore ==
* Brendan (BCOS)?
* Love4Boobies (Simplexity)
* '' "do I qualify, I wrote the original article" -- [[User:Mystran|Mystran]] 12:57, 1 April 2007 (CDT)''
* gravaera
* ethereal (Sydi)
* '' insert your names here ''


[[Category:Developer Archetype]]
[[Category:IPC]]
````

<a id="w001579"></a>

## W001579 — SimICS

Bron-ID: W001579
Namespace: Main (0)
Revisie: 29959 | Datum: 2026-02-10T23:22:00Z
Bron: https://wiki.osdev.org/SimICS
Revisie-URL: https://wiki.osdev.org/index.php?title=SimICS&oldid=29959

````text
{{stub}}

'''SimICS''' is an instruction-set simulator and operating system emulator for a variety of architectures, including PowerPC, Sparc, x86, MIPS and ARM. It is a powerful tool for computer architecture research, program debugging, and performance analysis of software systems.

There used to be an older version available for free; unfortunately this has now disappeared. The commercial version's site is http://www.windriver.com/products/simics/index.html (http://web.archive.org/web/20160506133614/http://www.windriver.com/products/simics/index.html) . University faculties can obtain a copy for free, see http://www.windriver.com/universities/.

[[Category:Emulators]]
````

<a id="w001580"></a>

## W001580 — James T. Klik

Bron-ID: W001580
Namespace: Main (0)
Revisie: 24652 | Datum: 2020-05-07T19:02:24Z
Bron: https://wiki.osdev.org/James_T._Klik
Revisie-URL: https://wiki.osdev.org/index.php?title=James_T._Klik&oldid=24652

````text
Look through the window ... See that background with alpha-blending and my anti-aliased fonts? See the corner of the screen? That's Klik's Start Menu! Hmm, no. I have no 'programs' folder, as I can't load 'programs' for now, but here's a 16-items list of test cases for my WidgetToolKit. Or maybe you prefer a [[What order should I make things in|different style]].

== Reaching Klik's level ==
=== Step 1: Documenting on video ===

# Read about [[VGA Resources]] and [[GRUB]]. You have the opportunity to let the bootloader switch to graphical mode for you before your kernel is started. A color depth of 32 will avoid you further headaches. [[:Category:Video|Documents]] about [[Accelerated Graphic Cards]] are gifts from the gods for Klik, but they're rare.
# Develop (or reuse) a small library to draw lines, fill areas, copy bitmaps and draw text '''out''' of your kernel, debug it, '''then''' integrate it in your OS.
# Find a friend to make a neat logo, ask GRUB to load it as if it was a module, memcpy it to screen and voilà. You can release 0.1

Note that you'll need [[Inline Assembly/Examples|inline assembly functions]] like everybody else, and that you'll probably prefer to have a "sprintf" than a "printf" function.

=== Step 2: Getting input ===

# Document about mouse control. Try to imagine a clean way of getting mouse clicks, position, events like drags, drops, slides, etc. A common engineering technique for this is the time-travelling-camera: move yourself in the future and imagine the user, then come back and draw on paper what you've seen.
# Grab a [http://www.gamedev.net/ game developing tutorial] about sprites, and see how you can have a moving object like a cursor. The trick is that you have to keep a small snapshot of what was '''under''' the cursor so that you can '''erase''' the cursor when it moves.
# Connect your cursor to your mouse and voilà. See it moving. By the same time, develop a small control widget that will report coordinates and events names (MouseDown, MouseOver, etc).

=== Step 3: Populating your screen ===

It is time to draw small windows on screen. We had many discussion in the forum about how they can be moved efficiently and so on. Note that you'll '''really''' start to need a way of [[Memory Management|allocating and freeing memory]] at this point, as you do '''not''' want to assume that there's a maximum of N buttons in the system ...

# Collect tutorials about GUI programming, including [http://osdever.net/tutorials/GUI_tut.php?the_id=62 GUI Programming] at BonaFide.
# Create a class that will be responsible for receiving event from the system, including the "repaint" event.
# If you want 'windows', let there be a window manager that send them 'repaint' events when needed, handle window displacement and is in charge of "what's behind the window".
# make sure you can add content to your window, and that you don't need to write a specific "repaint" function for each new content. Most WMs allow a window to be a container of widgets.

== Outrunning James T. Klik ==

=== Have programs loaded in memory ===
And provide a filesystem-approach to access them. It can be very easy if you use GRUB modules to map programs in memory: you just need to create a virtual directory service that will remember at which physical memory address the image of FILE.COM is located and return that address when we do open("file.com").

=== Have user-level programs executed when requested ===

The easiest approach is to create a segment for the user's code and one for its data. Then return to the program. You can even not worry about multitasking and decide every kernel operation will occur when an interrupt arise (that's how DOS works), but you will at least have '''1a. implement interrupts support''' and '''1b. implement basic segmentation support'''. If you're fine with asm, load asm programs as a first step.

=== Have the kernel access hardware ===

It should at least be able to receive the keyboard data (and translate them), setup video display and read mouse events. The most simple GUI at kernel level would be get_video_segment(), set_mouse_buffer(), set_keyb_buffer(). The kernel should know then where it should put data. You can even protect your "start menu" from the user application by excluding the last lines of the display from the "video segment" you return to the application.

Required steps are '''2a. video driver''' (get a frame buffer), '''2b. mouse driver''' (have a cursor moving on screen) and '''2c. keyboard reader''' (have keystrokes translated and written to some memory area).

=== Build the menu at run time ===

Read available programs from the virtual file system and build up the list of "start menu" with it. Ideally, this should be done by some programs external to the kernel, but let's say you do it in the kernel (at initialization) for now.

When the start menu is clicked, popup the list. When an item is selected, start the application and freeze the currently running one. When the task bar is clicked, unfreeze the selected application. When the "user area" is clicked, defer the click to the application (send it an event).
    
This requires '''3a. Process control''': select which program is currently running and '''3b. Process sleep & wakeup''': let the application decide that it have nothing to do but waiting something to happen and let the kernel return to the application only when something occurred.

Note that this will only offer you something like Mac OS 1.0 and that you'll certainly find yourself in troubles as soon as you'll try to add something like modular design, networking or multitasking...

== James' Bookshelf ==
Jumping right into graphics is frowned upon by many authors, and consequently he owns very few books on the subject. However, he does find some books like [[Books|"Operating Systems Design and Implementation" by Andy Tanenbaum]] interesting for some of the "prerequisites" of GUIs, like multitasking. Yet he did find some good documentation on the graphics systems, such as [http://www.amazon.com/Fundamentals-Interactive-Computer-Graphics-Programming/dp/0201144689 "Fundamentals of Interactive Computer Graphics" by James Foley] useful. And he has printed copies of all of VESA's VBE standards.

== James T. Klik opponents' position ==
If you're wondering how you'll implement the 'start button' and its menu right from the start, you're probably getting things by the wrong end... When you want to make a word processing program, you don't ask yourself how you will have icons for "save", "print", etc. in a bar that you can move across your window. You first ask yourself how you will represent text, how you will implement basic operations like extracting text from one paragraph and put text into another one, how you will deal with embedded objects like pictures, etc. or how you will store your text efficiently on a file. Basically, you'll work yourself like that slow horse in the pack if you don't have a good kernel with [[What order should I make things in|a different approach]] up first.

Also, many proponents of [[protected mode]] find VESA unattractive, since you need some sort of hook into the BIOS.

== People and OSes related to James T. Klik ==
* [[User:Combuster|Combuster]]
* [[User:Novayoshi|NovaYoshi]]
* [[User:osdeverr|osdeverr]]: "Me back in 2015... Now I blame myself for that TBH."
* [http://omarrx024.github.io omarrx024]
* [http://www.oby.ro/os/  Bogdan Ontanu, Solar_Os, GUI, Widget toolkit and running loaded PE from disk]
* '' insert your names here ''

[[Category:Developer Archetype]] 
[[Category:Graphical_UI]]
````

<a id="w001581"></a>

## W001581 — Nick Stacky

Bron-ID: W001581
Namespace: Main (0)
Revisie: 19878 | Datum: 2016-10-12T17:13:37Z
Bron: https://wiki.osdev.org/Nick_Stacky
Revisie-URL: https://wiki.osdev.org/index.php?title=Nick_Stacky&oldid=19878

````text
His test-machine has no keyboard and no screen. No one needs that. All he needs are NICs (Network Interface Cards) ... Plenty of NICs and Ethernet cable to see if his kernel responds to pings and routes packets correctly. His kernel of course has powerful multithreading and a complete TCP/IP stack. Or maybe you want to start with [[What order should I make things in|something else]].

== Nick Stacky's challenge ==
=== Accessing your NIC. ===

# First of all, buy one. A RTL8139 preferably or a [[ne2000]], as they're well documented with well-documented [http://cvs.sourceforge.net/viewcvs.py/mobius/coreos/drivers/ hobby-drivers] available for them. The [[Bochs]] has ISA NE2K support while Qemu has it as a PCI device. Another nice card is the [[3c90x]].
# Get as much documentation about those chips as you can, figure [http://beej.us/guide/bgnet/ how they work]
# Write/port tools that will scan the [[PCI]] bus for such device(s).

=== Sending packets ===

First thing, you should learn to send/receive packet on your own system. I strongly recommend the [http://beej.us/guide/bgnet/ Beej Programming Guide], and reading about encapsulation of protocol headers will be a must.

 +--*--*--*--*--*--*--*--+--*--*--*--*--*--+--*--*--*--*--*--+--*--+----/ /--+--*--*--*--+
 |  preamble & startbit  | destination add |   source addr   | Len | data    |    CRC    |
 +-----------8-----------+--------6--------+--------6--------+--2--+----/ /--+-----4-----+

Note that for technical reasons, the "data" field must be at least of 46bytes (though "len" may be smaller) and may not be of over 1500 bytes. NIC usually doesn't check this, but failing to do so will lead to packet lost or misinterpreted by other components.

Most NIC cards use the concept of ring buffers that contain packets received (but not yet transferred to main memory) or packet to send. The NIC issues an IRQ when a packet is available in such buffer or when a packet has been successfully emitted (check out the programming guide of your card for details).

Quoting the RTL8139(a/b) pguide, you have to

# copy the packet to a physically continuous buffer in (main) memory
# write the descriptor (start address, size, and miscellaneous control values) on the card's registers

In response, the NIC will

# starts moving data in its xmit FIFO and will start emitting on the wire as soon as it has "enough" data (defined by the early threshold on 8139)
# indicates it has the whole packet in FIFO by modifying the descriptor (setting a status bit somewhere)
# indicates it has emitted the packet by setting another status bit
# trigger an interrupt if interrupts have been enabled for this operation.

=== Supporting IP, TCP, UDP ===
{{Stub}}

== Going further than Nick Stacky ==

One difficulty for Nick is that with his all-wired OS, it can be quite difficult to change the machine setup. Here again, using one GRUB module that will contain the network configuration (IP address to use, address of the default gateway, routing table, etc.) may greatly help.

== Nick Stacky's Bookshelf ==
He owns a copy of [http://www.amazon.com/Computer-Networks-Approach-Kaufmann-Networking/dp/1558605142 "Computer Networks: A Systems Approach" by Larry L. Peterson & Bruce S. Davie] that he found by chance in a local bookstore. But he hates reading much else in the print side, since reading print is too slow. He'd rather download the book from his experimental network and gaze at its virtual beauty, then go back and do it again.

== Nick Stacky's Opponents position ==

A kernel should be able to do more than just answering pings. You have plenty of code following RFC, but you can't even display a single character on screen ... Is TCP really interresting if your kernel cannot even have programs that exchange data through those connections ? And don't talk me about your kernel-level POP/SMTP server when i talk to you about user programs ...

== People and OSes related to Nick Stacky ==
* Pcmattman: ''Even in the first version I had a fully functional UDP/IP stack, and I always find writing the card drivers easier than other things''
* [[User:01000101|01000101]]

* '' insert your names here ''


[[Category:Developer Archetype]]
````

<a id="w001582"></a>

## W001582 — Lino Commando

Bron-ID: W001582
Namespace: Main (0)
Revisie: 26660 | Datum: 2021-11-07T17:09:25Z
Bron: https://wiki.osdev.org/Lino_Commando
Revisie-URL: https://wiki.osdev.org/index.php?title=Lino_Commando&oldid=26660

````text
He's been impressed by the "naked beauty" of DOS/Unix times. The first thing he wants to be achieved is a blinking cursor after a ">" symbol so that he can type commands. No matter if there's no way to start 2 programs at the same time: he just needs a text editor and a FAT12 driver. Maybe you prefer a [[What order should I make things in|different approach]].

== Reaching Lino's level ==
=== A first shell ===

# Your first goal is an improved keyboard handler. Everything will start there. Make sure you leave room for a keyboard map (for instance easily compilable apart and loaded as a module by [[GRUB]] so that beta-testers can load their own map).
# Don't echo characters right from the start and keep track on what's on screen. Commands histories, completions and working character erasure will be a must.
# just put a few "built-in" commands from the start, but keep your mind open to "how will further shell extension add commands?"... scanning a table rather than a switch statement? you're the one who knows.

=== Starting programs ===

# Decide what programs will look like. The easiest model is to give them one data+code segment and let them run there.
# Your programs will have to come from somewhere. Lino had used one GRUB module as a '[[initrd|ramdisk]]' where programs were stored (he didn't tell whether using FAT or something even more simple for the internals). What is certain is that program see them as "real" files.
# Provide your programs a simple API: open/close/read/write to files, read/write to the console, exit are enough at the start. Prevent your programs from writing directly to video memory.

=== More things ===

Real shells have parameters, environments and such things which can be easily defined as read-only segments. If your favourite language doesn't like multi-segments, add a library that will copy strings in a buffer at request.

Once this is done ask yourself how you'll start other programs from one program.
This means implementing the [[System calls|system(8) call]].

== Going further than Lino ==
Well, in many aspects, Lino's OS looks like first steps of most Unix clones.

Because of the simple "start at offset 0 in CS/DS" approach, whether multiple programs share the same address space or not is not much complicated, but Lino still has to write [[Multitasking Systems|multitasking]] support, which will require him a more sophisticated way of [[Memory management|allocating and freeing memory]]

== Lino's bookshelf ==
Most books on the market will probably not be of interest (since they discuss "superfluous" things, like UNIX and multitasking). He'll probably look at [http://www.amazon.com/Emulator-Platform-Independence-Embedded-Development/dp/0879304367 "FreeDOS Kernel" by Pat Villani] and that would be enough. And sure, you can have some of the old volumes of MS-DOS reference manuals and infinite books by Dan Gookin lying around somewhere, don't you? And heck, why not, let's add [http://www.amazon.com/Inside-Windows-System-Nutshell-Handbook/dp/156592200X "Inside The Windows 95 File System" by Stan Mitchell] or Microsoft's [http://www.osdever.net/documents/fatgen103.pdf?the_id=40 FAT specification]. And yes, most importantly, is [http://www.amazon.com/Peter-Norton-Programmers-Guide-IBM/dp/0914845462 The Peter Norton Programmer's Guide to the IBM PC], which is appropriate for the time.

== Lino Commando's opponents position ==
The issue with Lino's approach is that he risks facing the same problems as DOS did: monotasking, too tight coupling between programs and hardware, etc. Despite how good Lino's OS project may be, he'll have to make sure that he studies sufficient abstractions right from the start and that he can't forget that one day he'll want more than one program to run.

== People and OSes related to Lino Commando ==
* Inflater: ''"That's me alright :) I just don't have fully working FAT12 driver for my real mode 16-bit Borland Pascal OS :D)"''
* Piranha: ''"Me too. Who needs graphics at this stage! Just making the internals work right is hard enough!"''
* Troy: ''"Text mode calls to me like bananas call to monkeys. Except with me there's no flinging of poo."''
* iMate900: ''Lino, here I come kicking myself with SauOS!!!! Oh, and I'm not even finished with my ATA driver :)''
* KotuxGuy: ''Yeah, this is most likely me too ;) I don't have any filesystem support though''
* bellezzasolo: ''This is always the best way to start. Nice if it's a PMode shell though ;)''
* LifeDJIK: ''Yes! That's definitely me! But I don't have FAT12 driver... I have FAT32 driver!''
* Moondeck: ''Graphics? VGA? Who needs that crap, I use serial!''
* Superleaf1995: ''Like me :D, I'm just doing with LeafOS, Why use graphics when you have a fancy text?''
* ''Killaship: Probably me (Minus the FAT-anything drivers), I just think text mode looks nice!''
* ''Insert your name here''

[[Category:Text UI]]
[[Category:Developer Archetype]]
````

<a id="w001584"></a>

## W001584 — FASM

Bron-ID: W001584
Namespace: Main (0)
Revisie: 25422 | Datum: 2020-12-10T16:20:30Z
Bron: https://wiki.osdev.org/FASM
Revisie-URL: https://wiki.osdev.org/index.php?title=FASM&oldid=25422

````text
The flat assembler (FASM) is a fast and efficient self-assembling [[x86]] assembler for DOS, Windows and *nix (Linux, BSD, MacOS etc.) operating systems. Currently it supports all x86/x64 instructions with MMX, 3DNow!, SSE up to SSE4, AVX, AVX2 and XOP extensions, can produce output in binary, MZ, PE, COFF or ELF format. It includes a powerful but easy to use macro language and compiles in multiple passes to optimize the instruction codes for size. The flat assembler is self-compilable and the full source code is included.

There's a version called '''FASM-ARM''' that generates AArch32 and [[AArch64]] native instructions (on x86, so this is a cross-compiler) but uses the same macro infrastructure as FASM.

The next generation of FASM is called '''FASMG''', which uses even more sophisticated macro infrastructure, and also macros to describe the instructions to be generated. Therefore it can be used to generate literally anything (macro definitions are available for x86, AArch64, Z80, MOS 6502, [[WebAssembly]], Java and Dalvik bytecode etc.)

==See Also==
===Articles===
*[[Assembly]]
*[[GAS]]
*[[NASM]]
===External Links===
*[https://www.flatassembler.net/ Official flat assembler website]
*[https://arm.flatassembler.net/ FASM-ARM cross-assembler]
*[https://board.flatassembler.net/topic.php?t=19389 FASMG official resources]

[[Category: Assemblers]]
[[de:Flat_Assembler]]
````

<a id="w001587"></a>

## W001587 — Talk:Books

Bron-ID: W001587
Namespace: Talk (1)
Revisie: 27429 | Datum: 2022-09-23T09:49:16Z
Bron: https://wiki.osdev.org/Talk:Books
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ABooks&oldid=27429

````text
Should we hide the TOC or try to restructure the template to get it to work with the TOC if possible or just leave as is? - [[User:Chase|Chase]] 23:39, 11 December 2006 (CST)

'''Hide''' I can't see it fitting in. -[[User:Jhawthorn|Jhawthorn]] 01:02, 12 December 2006 (CST)

: I think it should be hidden. --[[User:Walling|Walling]] 07:52, 12 December 2006 (CST)

== Category ==

Do you think we should put this into a category? --[[User:Stinkfly|Stinkfly]] 04:09, 24 August 2007 (CDT)
:'''No.''' It is already categorized, and it doesnt further categorize nicely with anything but an 'Anything' page (which we [[Main Page|already have]]). Also, if you meant separating this page into books: it makes things harder to find, as well as removing the ability to do searches over all the books at once (try Ctrl+F). - [[User:Combuster|Combuster]] 04:32, 25 August 2007 (CDT)

== Operating Systems Design and Implementation (3rd Edition) ==
The title says '(3rd Edition)', but the official description says "... Second Edition ...". The description should be updated to describe the 3rd edition. I was going to update it myself but I saw NOEDITSECTION at the top of the page - does that mean the page should not be edited?.
[[User:Steve23|steve23]] 08:41, 11 August 2009 (UTC)

== Revision ==

Is everyone okay with me completely rewriting this article which is, apparently, no longer maintained? For many of the books here there are many editions, there are no reviews---only the publisher's descriptions, there are no links for errata and/or supplementary material, there is no categorization, etc. I was working on a big revision for [http://forum.osdev.org/viewtopic.php?f=1&t=6989&p=174472#p174472 this forum post] when I figured this stuff would make much more sense here. I will obviously also add a link to the forum topic, in case anyone is interested. --[[User:Love4boobies|Love4boobies]] 05:19, 28 May 2012 (CDT)

: I'm a little bit uneasy with us including reviews of the books here.  I have several of these books and reasonably strong opinions about them but am reluctant to post them here because IMHO we are not a review site and a wiki is not an appropriate place for such things, given that anyone can change any review they don't agree with.  Something like the reviews section of Amazon is probably the better place for such personal opinions as it allows aggregation of scores and allows the community to up and down rate other reviews as they see fit.  Other than that I am all for people substantially revising old and unmaintained material. [[User:Jnc100|John]] 06:29, 28 May 2012 (CDT)

::One can still review books with some level of objectivity. I'm thinking of things like "this book covers X and Y but not Z," "this book requires advanced knowledge on topic X---better read Y first," "this book has factual errors and is thus not recommended," etc. Amazon reviews on technical books are often either misleading or not useful. --[[User:Love4boobies|Love4boobies]] 08:06, 28 May 2012 (CDT)

:::Apologies for late reply.  That seems okay, and unless anyone else has any objections please carry on. [[User:Jnc100|John]] 08:39, 5 June 2012 (CDT)

== Device Drivers ==
Would it make sense to include a section for books focused specifically on writing device drivers? [[User:Mfnickster|Mfnickster]] 04:49, 23 September 2022 (CDT)
````

<a id="w001588"></a>

## W001588 — Category:Processes and Threads

Bron-ID: W001588
Namespace: Category (14)
Revisie: 2597 | Datum: 2006-12-28T22:36:49Z
Bron: https://wiki.osdev.org/Category:Processes_and_Threads
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AProcesses_and_Threads&oldid=2597

````text
This category contains articles with information pertaining to Processes and Threads.

[[Category:OS theory]]
````

<a id="w001589"></a>

## W001589 — Template:Infobox Instruction

Bron-ID: W001589
Namespace: Template (10)
Revisie: 7020 | Datum: 2009-01-08T12:29:20Z
Bron: https://wiki.osdev.org/Template:Infobox_Instruction
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AInfobox_Instruction&oldid=7020

````text
<table align="right" style="width: 25em; text-align: left; font-size: 95%; border: 1px solid #aaa; background-color: #f9f9f9;">
<tr><td colspan="2" style="text-align: center; font-size: 150%;">'''{{{Instruction}}}'''</td></tr>
<tr><td colspan="2" style="text-align: center; font-size: 115%;">'''{{{Definition}}}'''</td></tr>
<tr><td colspan="2">&nbsp;</td></tr>
<tr><th width="150" style="vertical-align: top">Usage:{{If|{{{UsageSyntax|}}}|&nbsp;({{{UsageSyntax}}})}}</th><td><code>{{{Usage}}}</code></td></tr>
{{If|test={{{Usage2|}}}|then=<tr><th style="vertical-align: top">Usage:{{If|{{{Usage2Syntax|}}}|&nbsp;({{{Usage2Syntax}}})}}</th><td><code>{{{Usage2}}}</code></td></tr>}}
<tr><th>Modifies Flags:</th><td>{{{Flags|None}}}</td>
<tr><th>Availability:</th><td>{{{Availability|all}}}</td>
</table>
````

<a id="w001595"></a>

## W001595 — Template talk:Infobox Instruction

Bron-ID: W001595
Namespace: Template talk (11)
Revisie: 7021 | Datum: 2009-01-08T12:33:10Z
Bron: https://wiki.osdev.org/Template_talk:Infobox_Instruction
Revisie-URL: https://wiki.osdev.org/index.php?title=Template_talk%3AInfobox_Instruction&oldid=7021

````text
== Spelling ==

Just a quick question: It is called "usage" and not "useage", isn't it? --[[User:Walling|Walling]] 08:27, 13 December 2006 (CST)

"Usage" is correct - [[User:Combuster|Combuster]] 12:21, 13 December 2006 (CST)

: I fixed the spelling error in the template and where it is included. --[[User:Walling|Walling]] 18:15, 13 December 2006 (CST)

== Different Syntaxes? ==

Why only one "usage"?  A section for AT&T syntax would be nice.  Some important differences would be for long jumps, where AT&T and Intel syntax differ a lot.
:After some changing, debugging, etc... (pff) it works now: you may use multiple usages, and the syntaxt is like this:
:<pre>{{Infobox Instruction|Instruction=MOV|Definition=Some definition|Usage=MOV dest, src|UsageSyntax=Intel|Usage2=MOVL %src, %dest|Usage2Syntax=AT&T}}</pre>
:Which results in:
{{Infobox Instruction|Instruction=MOV|Definition=Some definition|Usage=MOV dest, src|UsageSyntax=Intel|Usage2=MOVL %src, %dest|Usage2Syntax=AT&T}}
:<div style="clear:both">[[User:Virtlink|Virtlink]] 12:33, 8 January 2009 (UTC)</div>
````

<a id="w001597"></a>

## W001597 — Talk:Networking

Bron-ID: W001597
Namespace: Talk (1)
Revisie: 7595 | Datum: 2009-04-06T22:55:27Z
Bron: https://wiki.osdev.org/Talk:Networking
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ANetworking&oldid=7595

````text
A TCP/IP stack is an interesting start, IMHO.[[User:Adek336|Adek336]] 06:44, 10 July 2008 (UTC)

This page is confusing - I intend to remove it since the category page offers pretty much the same information. - [[User:Combuster|Combuster]] 12:24, 13 December 2006 (CST)

: Yes, remove it. In the future one should write a general purpose networking article. Sort of "What do you want to know about networking for starters and experienced ... mixed with links to other articles with networking info or guides." Just an idea. :-) --[[User:Walling|Walling]] 18:39, 13 December 2006 (CST)

I'll throw together a rewrite of this page on my user page in the next couple of weeks --[[User:Pcmattman|pcmattman]] 02:48, 1 April 2009 (UTC)


Copied to [[Network_Stack]]. This page can probably be deleted now? --[[User:Pcmattman|pcmattman]] 03:49, 4 April 2009 (UTC)
:Euhm, there's a move button for this... Will make it a redirect - [[User:Combuster|Combuster]] 22:55, 6 April 2009 (UTC)
````

<a id="w001598"></a>

## W001598 — Loopback Device

Bron-ID: W001598
Namespace: Main (0)
Revisie: 20229 | Datum: 2017-01-23T00:55:43Z
Bron: https://wiki.osdev.org/Loopback_Device
Revisie-URL: https://wiki.osdev.org/index.php?title=Loopback_Device&oldid=20229

````text
A Loopback Device is a mechanism used to interpret files as real devices. The main advantage of this method is that all tools used on real disks can be used with a loopback device.

Note: This article only covers UNIX environments (including [[Cygwin]]). For information on how to use loopback devices on Windows, see [[Windows_Tools#diskpart | diskpart]].

== Loopback Device under Linux==
The linux loopback device can be used by root only, and needs to be enabled in the kernel before use.

=== Floppy Disk Images With GRUB and EXT2 ===

First, lets create an empty image.

 dd if=/dev/zero of=floppy.img bs=512 count=2880

Now, lets set it up for mounting.

 losetup /dev/loop0 floppy.img

Now lets make it EXT2 formatted.

 mkfs -t ext2 /dev/loop0

Mount!

 mount -t ext2 /dev/loop0 /mnt/myfloppy

Create GRUB directory.

 cd /mnt/myfloppy
 mkdir grub

Copy GRUB's second stage files. (GRUB stage[12] could also be located in /usr/lib/grub/)

 cp /lib/grub/i386-pc/stage[12] /mnt/myfloppy/grub

Create a device mapping for the GRUB installation. ''You need quotations around the first part.''

 echo "(fd0) /dev/loop0" > /mnt/myfloppy/grub/device.map

Start GRUB console for installation into the boot record.
 
 grub --device-map=/mnt/myfloppy/grub/device.map /dev/loop0

In the GRUB console:
 
 root (fd0)
 setup (fd0)

NOTE: You must unmount /mnt/myfloppy before using a emulator to directly read /dev/loop0, such as:

 qemu -fda /dev/loop0

NOTE: When deleting the loop device, the original floppy.img file will be saved with the modified contents.

=== Floppy Disk Images With FAT16 ===

Create an empty image.
 
 dd if=/dev/zero of=floppy.img bs=512 count=2880

Set it up for mounting.

 losetup /dev/loop0 floppy.img

Make it MSDOS formatted.

 mkdosfs /dev/loop0

Mount!

 mount -t msdos /dev/loop0 /mnt/myfloppy

=== Floppy Disk Images With FAT12 ===

The steps mentioned below will be useful for the BrokenThorn Entertainment tutorials

Create an empty image.
 
 dd if=/dev/zero of=floppy.img bs=512 count=2880

Set it up for mounting.

 losetup /dev/loop0 floppy.img

Format it to FAT12.

 mkdosfs -F 12 /dev/loop0

Mount!
 mount /dev/loop0 /mnt -t msdos -o "fat=12"

Unmount

 umount /mnt

Destroy loopback device

 losetup -d /dev/loop0

=== Hard Disk Images ===

A hard disk image contains an MBR, then a number of partitions, but the 'mount' instruction in Linux works with disk partitions, not full disks. To mount a partition contained in our disk image, we need to make sure the 'mount' command only sees our partition, not the whole disk.

==== Creating an image ====

First create the empty file that we will use for our disk image. We will assume a disk geometry of #cylinders, 16 heads, 63 sectors/track, 512 bytes/sector, which means that each cylinder contains 516096 bytes (16*63*512). Decide how large you want your disk image to be, and choose an appropriate number of cylinders (I’ll be using #cylinders throughout).

Example: If I want a 500Mb disk, I would choose 1000 cylinders (approximation of (500*1000*1024)/516096).

Write the disk image (I'll assume the filename c.img throughout):

 dd if=/dev/zero of=/path/to/c.img bs=516096c count=#cylinders

Explanation:
{| {{Wikitable}}
|-
| dd 
| Linux command for copy and convert a file
|-
| if=/dev/zero
| Source file is /dev/zero which is...*drumroll*...an infinite source of zeros
|-
| of=/path/to/c.img
| Destination file is our disk image (dd will create the file if it doesn't exist)
|-
| bs=516096c
| Means read and write 516096 bytes at a time (This is just here to keep things simple)
|-
| count=#cylinders
| Copy this number of blocks. Since we have set bs to 516096 bytes each block is one cylinder long
|}
That leaves us with a nice sized file full of zeros that we'll use for our disk image.

==== Partitioning ====

Now to create the MBR and partition table on the disk image (Usually you need to be root).

 fdisk -u -C#cylinders -S63 -H16 /path/to/c.img

Explanation:
{| {{Wikitable}}
|-
| fdisk 
| Linux DOS partition maintenance program.
|-
| -u 
| Display units in sectors not cylinders (We will need this).
|-
| -C#cylinders 
| Set the cylinders of disk to our value.
|-
| -S63 
| Set the sectors/track to 63.
|-
| -H16 
| Set the heads/track to 16.
|-
| /path/to/c.img 
| fdisk is capable of partitioning image files directly.
|}

Within fdisk use the following commands:

 o - Create a new empty DOS partition table.
 n - Create a new partition (For simplicity just make 1 primary partition covering the whole disk).
 a - Toggle the bootable flag (Optional).
 p - Print the partition table.

You should end up with a screen that looks something like this:

 Disk /path/to/c.img: 516 MB, 516096000 bytes
 16 heads, 63 sectors/track, 1000 cylinders, total 1008000 sectors
 Units = sectors of 1 * 512 = 512 bytes

       Device    Boot      Start         End      Blocks   Id  System
 /path/to/c.img1   *          63     1007999      503968+  83  Linux

Obviously the cylinder count, partition end and blocks will be different depending on the size of your image.

Make a note of the start sector (63 here) and the block count (503968 here).

''Note: If you are intending to format the partition to something other than ext2fs then change the partition id here using the t command. I should also point out that disk manufacturers and programmers don't agree on how many bytes are in a megabyte.''

 w - Write partition table to our 'disk' and exit.

Ignore any errors about rereading the partition table. Since it's not a physical device we really don't care.

We now have a partition table on our disk image.

Unfortunately this also means that from here on out we have to account for the fact that our partition does not start at byte 0 of the image.

==== Mounting ====

Ok, now we attach the file to the loopback device, in such a way that we skip everything before the start of our partition.

 losetup -o32256 /dev/loop0 /path/to/c.img

Explanation
{| {{Wikitable}}
|-
| -o32256 
| Move the start of data 32256 bytes into the file
|}

The reason we move 32256 bytes into the file is this is where the partition starts. Remember I said to note the start sector of the partition (63 is usual)? Well, since each sector is 512 bytes long we therefore know the starting byte of the partition is 32256 (63*512) bytes into the file. The reason behind this gap is that most (there is no real standard) fdisk programs don't use the first track for anything but the MBR. That space isn't always wasted though, some bootloaders (Eg GRUB) use it to store parts of their program.

''Note: If you aren't using the suggested geometry then you'll have to calculate this for yourself.''

We now have a device (/dev/loop0) which we can use in a similar fashion to a normal one for a partition (eg /dev/hda1).

==== Formatting the partition ====

For ext2fs, use:

 mke2fs -b1024 /dev/loop0 #blocks

Explanation:
{| {{Wikitable}}
|-
| mke2fs 
| Create an EXT2 filesytem
|-
| -b1024 
| Use block size of 1024
|-
| /dev/loop0 
| Device to make the filesystem on (Here /dev/loop0 is our 'partition')
|-
| #blocks 
| Remember I said to note the number of blocks from the fdisk section? This is why.
|}

This gives us a clean EXT2 formatted partition.

Note: mke2fs is smart enough to figure out block size and #blocks for itself, but if you ever want to use multiple partitions you'll need to know how to use those values.

For FAT32, use:

 mkdosfs -F32 /dev/loop0 #blocks

Explanation:
{| {{Wikitable}}
|-
| mkdosfs 
| Create a DOS filesystem (This may be absent on some Linux systems, search for the dosfstools package if it is)
|-
| -F32
| FAT 32 allocation tables (It should be obvious how to use FAT12/FAT16)
|-
| /dev/loop0 
| Same as for the ext2fs version
|-
| #blocks 
| Same as for the ext2fs version
|}

This gives us a clean FAT32 formatted partition (Ignore the floppy warning).

''Note: The reason for #blocks is the same as for ext2fs, ie possible multiple partitions.''

==== Mount Partition ====

You should now be able to mount the partition (Because it is still setup on the loopback device).

Command:

 mount -text2 /dev/loop0 /mnt/wherever

or:

 mount -tvfat /dev/loop0 /mnt/wherever

Explanation:
{| {{Wikitable}}
|-
| mount
| Linux command to mount a filesystem
|-
| -text2 / -tvfat
| Filesystem being used, Linux can usually figure this out on its own.
|-
| /dev/loop0
| The device representing our partition
|-
| /mnt/wherever
| A directory to mount the partition on.
|}

This should leave you with a nicely mounted partition. If you run df -Th you should end up with a line similar to:

 Filesystem    Type    Size  Used Avail Use% Mounted on
 /dev/loop0    vfat    492M  4.0K  492M   1% /mnt/wherever

...or for ext2fs:

 Filesystem    Type    Size  Used Avail Use% Mounted on
 /dev/loop0    ext2    477M   13K  452M   1% /mnt/wherever

(Yup, these are for the same disk image. By default ext2fs reserves/uses quite a bit of space even empty.)

==== Unmount, Detach ====

Ok, unmount the partition and detach the loopback device.

Command:

 umount /dev/loop0
 losetup -d /dev/loop0

Explanation:
{| {{Wikitable}}
|-
| umount
| Linux command to unmount a filesystem.
|-
| /dev/loop0
| The device that was mounted
|}

==== Making it Easier ====

One final thing to do, which is to simplify mounting and unmounting that partition.

Mounting:

 mount -text2 -oloop=/dev/loop0,offset=32256 /path/to/c.img /mnt/wherever

Unmounting:

 umount /path/to/c.img

This is essentially a combination of the losetup and mount commands we used previously when formatting the partition. If used it also means we lose access to the raw 'disk' or 'partition' through /dev/loop0.

See also http://www.pixelbeat.org/scripts/lomount.sh

Finally, if you have to mount and umount that image very frequently and you're too lazy to type the sudo password each time, just add to /etc/fstab:

 /path/to/c.img    /mnt/wherever    ext2    user,loop    0 0

now you can just call:

 mount /mnt/wherever
 umount /mnt/wherever

==== The End ====

That's it, you now know how to handle hard disk images under Linux. Whilst mounted you can use it in exactly the same way you use a normal disk partition. Multiple partitions are an extension of this, just change the offset of the losetup command according to the partition you want to use (And format using the correct number of blocks).

Things to remember:

* losetup type command will give you the equivalent of a raw disk device (Eg /dev/hda)
* losetup -o type command will give you the equivalent of a raw partition device (Eg /dev/hda1)

Don't forget to flush the filesystem buffers when manipulating with files on mounted disk image. On a Unix-like system, this can be simply done by executing the <tt>sync</tt> program in your shell.

== Loopback Device under FreeBSD ==

FreeBSD uses mdconfig. First, use DD to create an empty floppy image (1.44 MB in size). Memdisks are allocated dynamically, and the name is displayed after the mdconfig command. This assumes that "md0" is printed.

To mount: 
 dd if=/dev/zero of=floppy.img bs=512 count=2880
 mdconfig -a -t vnode -f floppy.img
 newfs_msdos -f 1440 /dev/md0
 mount -t msdosfs /dev/md0 /mnt/myfloppy

To unmount:
 umount /mnt/myfloppy
 mdconfig -d -u md0

== Loopback Device under OpenBSD ==

OpenBSD has used [http://www.openbsd.org/cgi-bin/man.cgi?query=vnconfig&apropos=0&sektion=0&manpath=OpenBSD+Current&arch=i386&format=html vnconfig(8)] since version 2.2 (perhaps earlier..).

As root or using su/sudo, Here is an example scenario for configuring a vnode pseudo disk device.

Creating the floppy.img file using dd:
 dd if=/dev/zero of=/path/to/floppy.img bs=512 count=2880

Configuring the vnd0 device:
 vnconfig vnd0 /path/to/floppy.img

Listing configured devices:
 vnconfig -l
 
 Output:
    vnd0: covering floppy.img on wd0a, inode 270473
    vnd1: not in use
    vnd2: not in use
    vnd3: not in use

Creating a FAT12 file system and then mounting the device:
 newfs_msdos -F 12 -f 1440 /dev/rvnd0c
 mount -t msdos /dev/vnd0i /mnt/floppy

Removing the device mount and uninstalling the vnd0 device:
 umount /mnt/floppy
 vnconfig -u vnd0

More Information: [http://www.openbsd.org/cgi-bin/man.cgi?query=vnd&apropos=0&sektion=4&manpath=OpenBSD+Current&arch=i386&format=html vnd(4)] / [http://www.openbsd.org/cgi-bin/man.cgi?query=vnconfig&apropos=0&sektion=0&manpath=OpenBSD+Current&arch=i386&format=html vnconfig(8)]

[[Category:Disk Image Utilities]]
````

<a id="w001599"></a>

## W001599 — Mkisofs

Bron-ID: W001599
Namespace: Main (0)
Revisie: 24395 | Datum: 2020-02-10T11:04:25Z
Bron: https://wiki.osdev.org/Mkisofs
Revisie-URL: https://wiki.osdev.org/index.php?title=Mkisofs&oldid=24395

````text
mkisofs is an utility that creates an [[ISO 9660]] image from files on disk.

From the man page for mkisofs,'' "mkisofs is effectively a pre-mastering program to generate the iso9660 filesystem - it takes a snapshot of a given directory tree, and generates a binary image which will correspond to an iso9660 filesystem when written to a block device."''

Developers of operating systems will mainly be interested in
creating ISO filesystems for bootable CD, DVD, or BD via [[El-Torito]].
Nevertheless, ISO filesystems may also be booted from hard disk or USB stick.

== Mkisofs, its Clone, and its Emulator ==
The program '''mkisofs''' is part of the
[http://cdrecord.org cdrtools project]
by Joerg Schilling.
<Source lang=bash>
prog="mkisofs"
</Source>

Some quarrels with the GNU/Linux community caused a fork of cdrtools,
named [http://www.cdrkit.org/ cdrkit]. The corresponding fork of mkisofs
is named '''genisoimage'''. You will find it installed as "mkisofs"
on many popular GNU/Linux distributions.
<Source lang=bash>
prog="genisoimage"
</Source>

An independent alternative to mkisofs is GNU's
[http://www.gnu.org/software/xorriso/ xorriso], which has an
emulation mode that understands the options which are used with
mkisofs for producing a bootable ISO 9660 image. Most distributions
have it either as package "xorriso" or as part of "libisoburn".
GRUB2 script grub-mkrescue depends on xorriso-0.5.6 or newer.
<Source lang=bash>
prog="xorriso -as mkisofs"
</Source>

Even the Linux distributions that ship genisoimage considers it
[https://wiki.debian.org/genisoimage outdated and bugged]. If you are
not using the UDF hybrid feature of genisoimage, you should
probably use xorriso on these systems instead. For EFI, xorriso is
mandatory.

'''Warning''': Defaults of '''xorriso -as mkisofs''' emulation might deviate from those of '''mkisofs'''.
E.g. up to xorriso version 1.4.8 not specifying '''-boot-load-size''' assumed a default of four sectors instead of the entire boot file.

== Options and input files ==
A mkisofs run needs one or more input directories or files
and a name for the emerging filesystem image file.
There are options for standards compliance, for filesystem
attributes, and -most important here- for setting up boot entry
points.

'''-o DISKPATH''' sets the result file name. E.g. "bootable.iso".

'''-R''' enables normal Unix filenames and attributes by
Rock Ridge extension. '''-r''' does the same but also changes
access permissions to make all files readable by everybody.

'''-J''' enables MS-Windows UCS-2 names via Joliet extension.

'''-iso-level NUMBER''' sets the ISO 9660 Level of Interchange.
Level 1 allows only 8.3 filenames. Level 2 allows up to 30
characters in filenames. Level 3 allows data files larger than
4 GiB - 1.

'''-U''' violates ISO 9660 specs by allowing long case-sensitive
filenames outside of said extensions. If you know that the reader
can stand it, then this is a cheap way to get decent file names.
But expect standard readers to become confused.

'''-V TEXT''' sets the ISO 9660 Volume ID. It can be up to 32
characters long and is often used by automounters as name of
the mount directory for the filesystem.

'''-hide ISOPATH''' and '''-hide-joliet ISOPATH''' may be used
to exclude a data file from the directory trees while still
having its content in data blocks of the filesystem.

'''-c ISOPATH''' sets the filename for the El Torito Boot Catalog.
The file is created automatically if boot images are announced.
If option -c is missing, then the file will not show up in the
directory trees of the ISO filesystem.

'''-b ISOPATH''' announces a data file as El Torito boot image
for PC-[[BIOS]].

'''-e ISOPATH''' announces a data file as El Torito boot image
for [[EFI]]. This is not an option of original mkisofs,
but is understood by some variants of genisoimage and
by xorriso -as mkisofs.

'''-no-emul-boot''' and '''-hard-disk-boot''' choose emulation
modes other than the default of floppy emulation. Especially
-no-emul-boot is needed for boot images of ISOLINUX and GRUB2.

'''-boot-load-size NUMBER''' sets how many blocks of the boot
image are to be loaded by the BIOS. A usual number with
-no-emul-boot is 4.

'''-boot-info-table''' causes a Boot Info Table to be written
into the boot image. Needed for boot images of ISOLINUX and GRUB2.

'''-eltorito-alt-boot''' ends the range of '''-b''' or '''-e'''.
All further El Torito boot options will apply to the next boot
image given by -b or -e.

'''-G DISKPATH''' copies the content of a data file into the
System Area of the ISO filesystem. This is used to bring [[MBR]]
or other boot blocks into the filesystem to enable booting from
hard-disk-like devices or booting on exotic hardware.

'''-isohybrid-mbr DISKPATH''' copies a SYSLINUX/ISOLINUX MBR
template into the System Area and makes it execute the El Torito
boot image for BIOS. This is a speciality of xorriso -as mkisofs.

'''-isohybrid-gpt-basdat''' announces the boot image as [[GPT]]
partition for booting via [[EFI]] and as MBR partition.
This is a speciality of xorriso -as mkisofs.

'''-graft-points''' enables the interpretation of pathspecs
which give a target path in the ISO filesystem together with
the path of an input directory or file.

A program argument, which is not recognized as option (e.g. because it
does not begin by a dash "-") and is not parameter of an option,
is taken as path to an input directory or file. If it is a
directory, then its content gets merged with the content of
the root directory of the emerging ISO filesystem. If it is
a file of other type, then it gets copied into the root directory.

If enabled, then pathspecs get recognized by an equal sign
which separates target path from source path. E.g.
 /boot/readme.txt=/home/me/texts/boot_iso_readme.txt
copies file
 /home/me/texts/boot_iso_readme.txt
into the ISO filesystem as
 /boot/readme.txt

== Hands-on examples ==
You want to create a bootable iso image?

First you need to build a bootable floppy image which holds f. ex. grub with your menu.lst and your kernel (and additional required modules).

Then, you put this boot floppy image in a temporary directory, say as /tmp/deploy/boot.img.

After that, you copy all your additional files (applications, images, texts, directories too) into the given directory (f. ex. /tmp/deploy).

Finally, you are ready to produce the ISO filesystem image
as /tmp/youriso.iso:

<source lang="bash">
$prog -U -b boot.img -hide boot.img \
      -V "Your Name..." -iso-level 3 -o /tmp/youriso.iso /tmp/deploy
</source>

Note that the ISOPATH for option '''-b''' is relative to the
root directory of the ISO filesystem. By providing the floppy image
as /tmp/deploy/boot.img you managed to let it appear in the ISO
filesystem as /boot.img.

For more examples see [[El-Torito]].

=== Under Windows ===
Windows version of mkisofs(through cdrtools) can be downloaded from ftp://ftp.berlios.de/pub/cdrecord/alpha/win32.
The following command(inside cygwin) can be used to create bootable iso image without using floopy image.
<source lang="winbatch">
mkdir -p $ISO_DIR/boot/grub
cp $GRUB_BIN/stage2_eltorito $ISO_DIR/boot/grub
cp menu.lst $ISO_DIR/boot/grub
cp your_kernel $ISO_DIR
 
mkisofs -R -b boot/grub/stage2_eltorito -no-emul-boot -boot-load-size 4 -boot-info-table -o bootcd.iso $ISO_DIR
</source>

== See Also ==
=== Articles ===
* [[ISO 9660]] - technical properties of ISO 9660 filesystems
* [[El-Torito]] - structure of bootable ISO 9660 filesystems with mkisofs examples
* [[Bootable El-Torito CD with GRUB Legacy]] - tutorial on how to create a "no emulation" bootable CD
* [[Bootable CD]] - tutorial on using floppy disk emulation
=== Forum ===
* [[topic:19050|Thread about making your own .iso image from scratch without tools]]
=== External Links ===
* [http://cdrtools.sourceforge.net/private/man/cdrecord/mkisofs.8.html  man 8 mkisofs] Manual of mkisofs
* [https://www.mankier.com/1/genisoimage man 1 genisoimage] Manual of genisoimage
* [https://www.mankier.com/1/xorrisofs man 1 xorrisofs] Manual of xorriso's emulation of mkisofs

[[Category:Disk Image Utilities]]
````

<a id="w001600"></a>

## W001600 — Hdiutil

Bron-ID: W001600
Namespace: Main (0)
Revisie: 11353 | Datum: 2011-04-05T04:03:19Z
Bron: https://wiki.osdev.org/Hdiutil
Revisie-URL: https://wiki.osdev.org/index.php?title=Hdiutil&oldid=11353

````text
hdiutil is the program supplied by Apple to work with disk images

== Support ==
hdiutil supports the following filesystem types:

* HFS+
* HFS+J
* HFSX
* HFS
* MS-DOS
* UFS

== Creating Images ==
Create a DOS disk image:

 hdiutil create -fs ms-dos -sectors 2880 floppy

(2880 sectors = 1.44mb floppy disk image)

If the image already exists, you must pass the -ov option or hdiutil will fail.

To create no apple partition image, use:

 -layout NONE

To create an image with an apple partition layout, use:

 -layout SPUD

I don't know if images created here are straight binary images that can be plugged into Bochs/VMWare etc...

* Example of creating and mounting an image:*

 hdiutil create -fs MS-DOS -sectors 2880 floppy

This will output floppy.dmg. Note that the MS-DOS argument is case sensitive.

 hdid -nomount ./floppy.dmg

This will output the name of the device file created.

 mkdir /Volumes/mnt
 mount -t msdos /dev/disk2 /Volumes/mnt

Use the name of the device file output in the previous step.

If you chose to not place your executable on a disk directly, you can use hdiutil burn ./floppy.dmg. If you try to use an ordinary burn, the boot image will be corrupt.
[[Category:Disk Image Utilities]]
````

<a id="w001601"></a>

## W001601 — File Disk

Bron-ID: W001601
Namespace: Main (0)
Revisie: 12043 | Datum: 2011-09-30T06:25:03Z
Bron: https://wiki.osdev.org/File_Disk
Revisie-URL: https://wiki.osdev.org/index.php?title=File_Disk&oldid=12043

````text
This little tool by Bo Brantén allows the mounting of a disk image to a drive letter. It works on WinNT/2000/XP/Vista/7. It can be found [http://www.acc.umu.se/~bosse/ here]

== Usage ==

Mounting a image:
Filedisk requires windows style full path of the image to mount the image. If a relative path or unix style path are given, it will fail.
Note: Inside cygwin, Unix style path name can be converted to Windows absolute path name using the command '''cygpath'''
   filedisk /mount 0 "C:\Full\Path\To\Image\disk.img" m:
   filedisk /mount 0 `cygpath -a -w Image/disk.img` m:

Unmounting a image:
   filedisk /umount m:

[[Category:Disk Image Utilities]]
````

<a id="w001602"></a>

## W001602 — MTools

Bron-ID: W001602
Namespace: Main (0)
Revisie: 25359 | Datum: 2020-11-14T05:13:24Z
Bron: https://wiki.osdev.org/MTools
Revisie-URL: https://wiki.osdev.org/index.php?title=MTools&oldid=25359

````text
Mtools is a collection of tools to allow Unix systems to manipulate MS-DOS based filesystems, such as [[FAT]]. It provides command-line utilities to read, write, and move files on an MS-DOS filesystem. Mtools is not capable of facilitating mounting an MS-DOS disk image to the host system. One common use for Mtools is to copy files to a FAT disk image located on the host system.

Mtools is available in the package repositories of most Linux distributions, or can be built from source under [[Cygwin]]. The source code is available here: https://www.gnu.org/software/mtools/

==Using disk images==
The <tt>-i</tt> option, followed by a disk images file name, allows for using a disk image in place of a real device when the drive letter <tt>:</tt> is used.

For example to copy the file <tt>kernel.elf</tt> from the working directory to the root directory of the image <tt>floppy.img</tt> one would use
  mcopy -i floppy.img kernel.elf ::/


It is important to notice that, while MTools claims to include only the minimal BIOS Parameter Block (BPB), it actually uses the FAT16 extension (see e.g. [http://www.win.tue.nl/~aeb/linux/fs/fat/fat-1.html] under 'FAT16' for an explanation) to the BPB if you format a FAT12 image.

==See Also==
===External Links===
* https://www.gnu.org/software/mtools/ Mtools home page.
* https://linux.die.net/man/1/mtools Mtools Linux man entry.


{{Stub}}

[[Category:Disk Image Utilities]]
````

<a id="w001603"></a>

## W001603 — WinImage

Bron-ID: W001603
Namespace: Main (0)
Revisie: 28076 | Datum: 2023-07-09T20:48:31Z
Bron: https://wiki.osdev.org/WinImage
Revisie-URL: https://wiki.osdev.org/index.php?title=WinImage&oldid=28076

````text
{{Stub}}

Win Image gives you an Explorer-like window on a disk image. You can create images, browse directories, and drag and drop files. Win Image supports images for FAT floppies and hard disks, and CD-ROMs (ISOs). It will also extract an image from a disk, or write an image to a disk. One drawback is that it's shareware, unlike File Disk and MTools, so if you want to use all of its features after the evaluation period expires, you will need to pay.

It can be found [http://www.winimage.com/ here]

[[Category:Disk Image Utilities]]
[[Category:Windows]]
````

<a id="w001604"></a>

## W001604 — BFI

Bron-ID: W001604
Namespace: Main (0)
Revisie: 28809 | Datum: 2024-05-13T10:51:12Z
Bron: https://wiki.osdev.org/BFI
Revisie-URL: https://wiki.osdev.org/index.php?title=BFI&oldid=28809

````text
BFI (Build Floppy Image) creates FAT12-formatted floppy images from the command line, inserting the user's bootsector and files. It is created using the [[Win Image]] SDK. BFI is available [http://www.nu2.nu/bfi/ here] ([https://web.archive.org/web/20090301055639/http://nu2.nu/bfi/ archive])

{{Stub}}

[[Category:Disk Image Utilities]]
````

<a id="w001605"></a>

## W001605 — Virtual Floppy Drive

Bron-ID: W001605
Namespace: Main (0)
Revisie: 30109 | Datum: 2026-03-03T15:05:08Z
Bron: https://wiki.osdev.org/Virtual_Floppy_Drive
Revisie-URL: https://wiki.osdev.org/index.php?title=Virtual_Floppy_Drive&oldid=30109

````text
Virtual Floppy Drive mounts a floppy disk image in a similar fashion to using a UNIX loop device as a normal floppy disk. You can then access the disk image by all normal means including disk editors (of course, not fdisk). It runs under Windows NT/2000/XP and possibly also 2003 and Vista (32-bit only, 64-bit requires extra compilation, see the link to the homepage below).

== Usage ==
This tool is used similarly to filedisk. After installing and starting the device driver with:

 vfd.exe install /AUTO
 vfd.exe start

You can mount and unmount the floppy using:

 vfd.exe mount floppy_image.img /L:b
 vfd.exe unmount

where /L:b instructs the driver to use drive B: for the virtual floppy.

== VFD 2.1 ==
In this newer version, the GUI has been expanded and VFD can be integrated into the Windows shell. This way, by assigning the application to the correct file types, you can simply double-click the floppy image and VFD will take care of the rest.

== Links ==
* [http://vfd.sourceforge.net/ homepage]

[[Category:Disk Image Utilities]]
[[Category:Virtualization]]
````

<a id="w001606"></a>

## W001606 — Disk Images

Bron-ID: W001606
Namespace: Main (0)
Revisie: 26884 | Datum: 2022-01-24T01:55:20Z
Bron: https://wiki.osdev.org/Disk_Images
Revisie-URL: https://wiki.osdev.org/index.php?title=Disk_Images&oldid=26884

````text
The best way for hobbyist os developers to test their systems is to use programs like [[QEMU]], [[Bochs]], [[VMware]] or [[Microsoft Virtual PC|Virtual PC]].

All of these common tools rely on using disk images (either floppy or hard disk). A disk image is nothing more than a file whose content reflects the layout of a disk and that the tool will use as if it was a physical device. E.g. A floppy image typically is 1.44MB large and its 512 first bytes consist of the 'floppy's boot sector. The different sub-pages should tell you how to create such images from real disks and use them under the most common operating systems.


== Tools ==

These programs can help you with the management of disk images
{| {{Wikitable}}
|- 
| 
! Linux
! Windows
! BSD
! Mac OS X
! [[:Category:Filesystems|File Systems]] Supported

|-
| [[BFI]]
| {{No}}
| {{Yes}}
| {{No}}
| {{No}}
| [[FAT]]

|-
| [[Fat_imgen|fat_imgen]]
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| [[FAT12]]

|-
| [[File Disk]]
| {{No}}
| {{Yes}}
| {{No}}
| {{No}}
| Anything supported by Windows

|-
| [[FUSE]] <sup>[[#Note1|1]]</sup>
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| Literally anything, including your own FS

|-
| [[hdiutil]]
| {{No}}
| {{No}}
| {{No}}
| {{Yes}}
| [[HFS]] HFS+ HFS+J HFSX [[FAT]] [[UFS]]

|-
| [[ImDisk]]
| {{No}}
| {{Yes}}
| {{No}}
| {{No}}
| Anything supported by Windows

|-
| [[Loopback Device]]
| {{Yes}}
| {{No}}
| {{Yes}}
| {{Yes}}
| Anything supported by the kernel

|-
| [[MagicISO]]
| {{No}}
| {{Yes}}
| {{No}}
| {{No}}
| ISO 9660, Rock Ridge, HFS/HFS+, Joilet, UDF, XBOX DVD FS

|-
| [[mkisofs]]
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| [[ISO 9660]]

|-
| [[MTools]]
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| [[FAT]]

|-
| [https://www.fysnet.net/ultimate/index.htm Ultimate]
| {{No}}
| {{Yes}}
| {{No}}
| {{No}}
| [[MBR]], [[GPT]], [https://www.fysnet.net/fysos_embr.htm eMBR], [[ISO 9660]], [[FAT]], [[ExFAT]], [[NTFS]], [https://www.fysnet.net/leanfs/index.php LEAN], [https://www.fysnet.net/fysfs.htm FYSFS], [[SFS]], [[Ext2]]

|-
| [[Virtual Floppy Disk]]
| {{No}}
| {{Yes}}
| {{No}}
| {{No}}
| Anything supported by Windows

|-
| [[Win Image]]
| {{No}}
| {{Yes}}
| {{No}}
| {{No}}
| [[FAT]] [[ISO 9660]]

|}

<small id="Note1">Note 1: FUSE is not an application, but a shared library and a set of applications; one for each file system. You can also write your own if you'd like. They are easy to use as they mount a disk partition or image file on a directory, and you can manipulate the files in the image with standard tools afterwards.</small>


If you want to write/burn an image to media you can use one of the following tools:
{| {{Wikitable}}
|- 
| 
! Linux
! Windows
! BSD
! Mac OS X
! Notes

|-
| dd
| {{Yes}}
| {{Yes}} (Using [[Cygwin]])
| {{Yes}}
| {{Yes}}

|-
| [[Rawwrite]]
| {{No}}
| {{Yes}}
| {{No}}
| {{No}}
| Floppy images only

|-
| [https://gitlab.com/bztsrc/usbimager/ USBImager]
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| very small, dependency-free and has a simple GUI

|}

== Floppy disk images ==

You can use 'dd' to create a blank floppy image.

<source lang="bash">
dd if=/dev/zero of=floppy.flp bs=512 count=2880
</source>

== CD images ==

As of version 0.95, GNU [[GRUB]] comes with support for no-emulation [[El-Torito]] CD boot. Creating a CD image is much easier than working with floppy images and trying to stuff GRUB in them (and you get 650 meg more space too). Putting GRUB on a CD is now a simple matter of making a skeleton directory tree for the CD filesystem layout, copying the "stage2_eltorito" file in there, and running mkisofs with a specialized command line. See the GRUB 0.95 info node [http://www.gnu.org/software/grub/manual/html_node/Making-a-GRUB-bootable-CD_002dROM.html Installation > Making a GRUB bootable CD-ROM] for details. Much more humane than those floppy games.

There is also a tutorial about creating a [[Bootable El-Torito CD with GRUB Legacy|no-emulation El-Torito CD with GRUB]].

== Handling of Partition Tables in images. ==

At the time of writing, only Apple's [[hdiutil]] and Plan 9's [http://9p.io/magic/man2html/8/partfs partfs] can handle the mounting of partitions inside of an image cleanly. Linux users can use [[Loopback Device|a hack]] to skip over the MBR sector. In general, you can solve the partitioning information problem by using a separate image for one partition, then insert it into a larger image with partitioning data at the right place. This works because offsets are generally relative to the start of the partition.

There's a tutorial on [[Bootable Disk]]s.

== Images with preinstalled GRUB ==

If you are looking for a ready-made floppy image with GRUB already installed, such has been set up by MartinBaute, and is available from the following mirrors:

* LibOSDK project at http://download.berlios.de/libosdk/fd.img.bz2 (GRUB 0.96) [broken link]
* Clicker project at http://sourceforge.net/projects/clicker/files/miscellaneous/GRUB%20bootdisk%20(by%20Solar)/solar_grub_disk.zip/download (GRUB 0.95)
* Ubuntu "grub-rescue-pc" package at http://packages.ubuntu.com/trusty/i386/grub-rescue-pc/download (GRUB2 2.00) [broken link]

The images have GRUB stage1 / stage2 and an empty config file set up, so all you have to do is adding your binaries and editing the config file (see the [http://www.gnu.org/software/grub/manual GRUB manual] for details).

The CD/DVD image on rdos.net has an embedded bootable floppy image and the required files in the root directory.

The Ubuntu "grub-rescue-pc" package is GRUB2, and has floppy, CDROM, and USB images.

== Pre-made Images to test your code with ==

The following is a list of pre-made images of various partitioning schemes and file systems.

The purpose of this list is to include images for testing your partitioning and file system code.  In other words, can your code find and mount the given partitions/file systems on these images.

This is not a 'vanity' list, a list to simply add your image file.  The ideal purpose of this list is for images with intentionally numerous partitions and/or partitioning schemes, and other types of images.  However, images with single partitions are allowed.  Please make sure--to the best of your knowledge--the partitioning scheme and file systems are correctly implemented before adding to this list.  The intent is for others to '''test their code on your image''', not to '''test your image with their code.'''

You are welcome to add your image URL, but please keep in mind the purpose of this list.

* Please compress the image and point the URL to the compressed image.  ZIP, GZ/BZ, etc.
* Please do not include many other files.  Keep it to the disk image itself.  If you wish to include the source, place the source on the disk image itself.
* Please use (somewhat) permanent links.  Do not add your image file if the URL will be broken in the near future.

{| {{Wikitable}}
|-
| URL/Name
! Partition Scheme
! Sector Size
! File System(s)
! Compressed
! Uncompressed
! Notes

|-
| [https://www.fysnet.net/ultimate/demodisk.zip Ultimate Test]
| [https://www.fysnet.net/fysos_embr.htm eMBR]
| 512
| [[FAT]], [[ExFAT]], [https://www.fysnet.net/leanfs/index.php LEAN], [https://www.fysnet.net/fysfs.htm FYSFS], [[SFS]], [[Ext2]]
| 5.5 Meg
| 165 Meg
| Test image for [https://www.fysnet.net/ultimate/index.htm Ultimate] disk viewer

|-
| [https://www.fysnet.net/zips/fysos.zip fysos]
| [[GPT]], floppy
| 512
| [[FAT]]
| 8.2 Meg
| 10 Meg/1.44 Meg
| Single FAT 16 file system partition with a GPT partitioning Scheme

|-
| [https://www.fysnet.net/ultimate/four_k_sects.zip Ultimate Test (4k)]
| [[GPT]]
| 4096
| [[FAT]], [https://www.fysnet.net/leanfs/index.php LEAN]
| 8.6 Meg
| 86 Meg
| FAT 16 & Lean file systems partitioned with a GPT partitioning Scheme (Using 4k Sector sizes)
Boot EFI only.  Legacy will not read 4k sector sizes.

|}

== Links ==

* Related thread in the forum: [[Topic:10549]] You gonna find a nifty trick Brendan uses to build his own iso images - with NASM. (download missing)
* John Burger's [[User:Johnburger/Demo|Demonstration]] uses [[User:Johnburger/Demo/Pad/ISO/ISO|NASM source code]] to build his own ISO images too.

[[Category:FAQ]]
````

<a id="w001607"></a>

## W001607 — Category:Disk Image Utilities

Bron-ID: W001607
Namespace: Category (14)
Revisie: 6455 | Datum: 2008-10-28T08:53:39Z
Bron: https://wiki.osdev.org/Category:Disk_Image_Utilities
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ADisk_Image_Utilities&oldid=6455

````text
This is a list of utilities that can be used to manipulate disk images. For a comparison read [[Disk Images]].

[[Category:Tools]]
````

<a id="w001608"></a>

## W001608 — Talk:Disk Images

Bron-ID: W001608
Namespace: Talk (1)
Revisie: 4519 | Datum: 2007-09-04T20:29:21Z
Bron: https://wiki.osdev.org/Talk:Disk_Images
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ADisk_Images&oldid=4519

````text
== Move ==
Should I move the list of utilities to [[Tool Comparison]] - [[User:Combuster|Combuster]] 14:16, 13 December 2006 (CST)

: I don't know. If you do it then make a link to it from this article so it is easily found. --[[User:Walling|Walling]] 18:18, 13 December 2006 (CST)

== About rawwrite(win), dd, nero, etc ==
I think image uploading/downloading utilities should be separated, if not removed from the comparison page - [[User:Combuster|Combuster]] 08:39, 11 March 2007 (CDT)
: That makes sense, but we maybe should not separate them on too long of a distance. Keep them cross-linked. The tools are so similar I did not realize it when adding Rawrite to the list, which also shows my limited mental capacity for editing. [[User:kmcguire|kmcguire]] 10:40 AM CST 11 March 2007

== Dead link ==
I marked the link to making a grub CD-ROM as dead [[User:Meshounah|Meshounah]] 11:25, 2 September 2007 (CDT)
:The link wasn't dead - it was an error in the wiki markup that broke the link. Fixed - [[User:Combuster|Combuster]] 15:29, 4 September 2007 (CDT)
````

<a id="w001609"></a>

## W001609 — User:Walling/Archive/Sandbox

Bron-ID: W001609
Namespace: User (2)
Revisie: 2299 | Datum: 2006-12-16T19:58:55Z
Bron: https://wiki.osdev.org/User:Walling/Archive/Sandbox
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AWalling%2FArchive%2FSandbox&oldid=2299

````text
<div id="mainpage"></div> __NOTOC__ __NOEDITSECTION__

<!-- Welcome to OSDev.org box -->
{{/BoxWelcome}}

<!-- Conversion from OSFAQ wiki to MediaWiki box -->
{{/BoxConversion}}

<!-- Table stuff -->
{| style="width: 100%; margin: 0; padding: 0; border: 0; border-collapse: collapse;"
| style="padding: 0; width: 25%; vertical-align: top;" |

<!-- First column -->
{{/Box|subject= OS development}}
{{/Box|subject= Resources }}

| style="padding: 0 0 0 10px; width: 25%; vertical-align: top;" |

<!-- Second column -->
{{/Box|subject= Languages |more=yes}}
{{/Box|subject= Tools |more=yes}}

| style="padding: 0 0 0 10px; width: 25%; vertical-align: top;" |

<!-- Third column -->
{{/Box|subject= Hardware |more=yes}}

| style="padding: 0 0 0 10px; width: 25%; vertical-align: top;" |

<!-- Forth column -->
{{/Box|subject= OS theory }}
{{/Box|subject= This site }}

|}
````

<a id="w001611"></a>

## W001611 — User:Walling/Archive/Sandbox/Box

Bron-ID: W001611
Namespace: User (2)
Revisie: 2301 | Datum: 2006-12-16T19:59:08Z
Bron: https://wiki.osdev.org/User:Walling/Archive/Sandbox/Box
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AWalling%2FArchive%2FSandbox%2FBox&oldid=2301

````text
<div style="margin: 0 0 10px 0; padding: 0 1em 0.7em 1em; background-color: #ffffff; color: #000000; border: 2px solid #dddddd;"><span class="plainlinks" style="float: right; margin-top: 0.5em;"><small>[{{fullurl:Template:{{{subject}}}|action=edit}} edit]</small></span>
{{{{{subject}}}}}
{{If|test={{{more|}}}|then=<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:{{{subject}}}|more...]]</small></div>}}
</div>
````

<a id="w001613"></a>

## W001613 — User:Walling/Archive/Sandbox/BoxConversion

Bron-ID: W001613
Namespace: User (2)
Revisie: 2303 | Datum: 2006-12-16T19:59:17Z
Bron: https://wiki.osdev.org/User:Walling/Archive/Sandbox/BoxConversion
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AWalling%2FArchive%2FSandbox%2FBoxConversion&oldid=2303

````text
<div style="margin: 0 0 15px 0; padding: 0.2em; background-color: #ffffdd; color: #000000; border: 2px solid #dddddd; text-align: center;">
'''We are in the process of organizing and converting the old [[mtfaq:|OSFAQ wiki]] to this MediaWiki.'''<br>
'''Please help if you can, see [[OSDevWiki:Pages To Convert|Pages To Convert]] for more details.'''
</div>
````

<a id="w001614"></a>

## W001614 — User:Walling/Archive/Sandbox/BoxWelcome

Bron-ID: W001614
Namespace: User (2)
Revisie: 2305 | Datum: 2006-12-16T19:59:26Z
Bron: https://wiki.osdev.org/User:Walling/Archive/Sandbox/BoxWelcome
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AWalling%2FArchive%2FSandbox%2FBoxWelcome&oldid=2305

````text
<div style="margin: 0 0 15px 0; padding: 1px; border: 2px solid #dddddd;">
{| style="width: 100%; margin: 0; padding: 0; border: 0; background-color: #ffffff; color: #000000; border-collapse: collapse;"
| align="left" style="vertical-align: top; white-space:nowrap;" |
<div class="plainlinks" style="width: 30em; text-align: center; padding: 0.7em 0;">
<div style="font-size: 220%;">Welcome to [http://www.osdev.org/ OSDev.org]</div>
<div style="font-size: 90%; margin-top: 0.7em; line-height: 130%;">This website provides information about the creation of<br>operating systems and serves as a [http://www.osdev.org/phpBB2/ community] for those<br>people interested in OS creation. [[Special:Statistics|{{NUMBEROFARTICLES}}]] wiki articles.</div>
</div>
| align="left" style="width: 10em; text-align: left; vertical-align: top; white-space:nowrap;"|
* [[OSDevWiki:About|About this site]]
* [[OSDevWiki:Editing|Editing help]]
* [[Projects|Project list]]
* [[:Category:FAQ|OS FAQ]]
|}
</div>
````

<a id="w001622"></a>

## W001622 — Template:OS Theory

Bron-ID: W001622
Namespace: Template (10)
Revisie: 2175 | Datum: 2006-12-14T01:59:08Z
Bron: https://wiki.osdev.org/Template:OS_Theory
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AOS_Theory&oldid=2175
Doorverwijzing: Template:OS theory

````text
#REDIRECT [[Template:OS theory]]
````

<a id="w001624"></a>

## W001624 — OSDev Wiki:About

Bron-ID: W001624
Namespace: OSDev Wiki (4)
Revisie: 29813 | Datum: 2025-12-30T01:35:35Z
Bron: https://wiki.osdev.org/OSDev_Wiki:About
Revisie-URL: https://wiki.osdev.org/index.php?title=OSDev_Wiki%3AAbout&oldid=29813

````text
Welcome to '''OSDev.org''', the largest online community of operating system developers. If you want to learn how to write your own OS we have all the information to get you started. Read our [[Main Page|OS development wiki]] to learn where to start. The [http://forum.osdev.org forums] are a great place to discuss OS theory and ask for help when you get stuck. Don't forget to add a link on the [[Projects|OS List]] to your OS project once it gets going.

This website provides information about the creation of operating systems and serves as a community for those people interested in OS creation. The wiki contain articles about various OS developing subjects. For more information on the history of this site, and OSDeving in general, see the [[History]] page.

== License of the text on the wiki ==

Copyright and licensing issues are being worked out for the moment. Read [[topic:13810|this thread on the forum]]. Also read [[OSDev Wiki:License|about the license here]] and the [[OSDev Wiki:General disclaimer|general disclaimer]].

== Offline archive ==

You can download an HTML copy of the wiki for offline viewing: https://files.osdev.org/osdev_wiki.zip

You can download an XML dump of the wiki (suitable for importing into your own MediaWiki): https://files.osdev.org/osdev-wiki.xml.gz

The wiki archives are automatically generated on a weekly basis.

==Software==
The forums run phpBB 3.3 and the wiki runs MediaWiki 1.39.7 LTS.

The PluggableAuth and PHPBB_Auth extensions are used to integrate the wiki with the forum account. See [[Special:Version]] for a list of MediaWiki extensions installed.

==Discuss the wiki==

If you want to discuss issues about the wiki go to the [[forum:8|OSDev Wiki forum]].

[[Category:Wiki administration|{{PAGENAME}}]]
````

<a id="w001625"></a>

## W001625 — User:Walling/Archive/Sandbox2

Bron-ID: W001625
Namespace: User (2)
Revisie: 2307 | Datum: 2006-12-16T20:00:58Z
Bron: https://wiki.osdev.org/User:Walling/Archive/Sandbox2
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AWalling%2FArchive%2FSandbox2&oldid=2307

````text
<div id="mainpage"></div> __NOTOC__ __NOEDITSECTION__

<!-- Welcome to OSDev.org box -->
{{User:Walling/Sandbox/BoxWelcome}}

<!-- Conversion from OSFAQ wiki to MediaWiki box -->
{{User:Walling/Sandbox/BoxConversion}}

<!-- Table stuff -->
{| style="width: 100%; margin: 0; padding: 0; border: 0; border-collapse: collapse;"
| style="padding: 0; width: 25%; vertical-align: top;" |

<!-- First column -->
{{/Box|subject= OS development}}
{{/Box|subject= Resources }}

| style="padding: 0 0 0 10px; width: 25%; vertical-align: top;" |

<!-- Second column -->
{{/Box|subject= Languages |more=yes}}
{{/Box|subject= Tools |more=yes}}

| style="padding: 0 0 0 10px; width: 25%; vertical-align: top;" |

<!-- Third column -->
{{/Box|subject= Hardware |more=yes}}

| style="padding: 0 0 0 10px; width: 25%; vertical-align: top;" |

<!-- Forth column -->
{{/Box|subject= OS theory }}
{{/Box|subject= This site }}

|}
````

<a id="w001626"></a>

## W001626 — User:Walling/Archive/Sandbox2/Box

Bron-ID: W001626
Namespace: User (2)
Revisie: 2309 | Datum: 2006-12-16T20:01:15Z
Bron: https://wiki.osdev.org/User:Walling/Archive/Sandbox2/Box
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AWalling%2FArchive%2FSandbox2%2FBox&oldid=2309

````text
<div style="margin: 0 0 15px 0; padding: 0; border: 0; background-color: #ffffff; color: #000000;">
{{{{{subject}}}}}

<small class="plainlinks" style="padding-left: 1em;">'''[{{fullurl:Template:{{{subject}}}|action=edit}} edit]''' {{If|test={{{more|}}}|then=&#73; '''[[:Category:{{{subject}}}|more...]]'''}}</small>
</div>
````

<a id="w001627"></a>

## W001627 — Ne2000

Bron-ID: W001627
Namespace: Main (0)
Revisie: 27691 | Datum: 2023-03-18T08:54:41Z
Bron: https://wiki.osdev.org/Ne2000
Revisie-URL: https://wiki.osdev.org/index.php?title=Ne2000&oldid=27691

````text
The '''Ne2000 network card chipset''' was a reference design that was never meant to go into mainstream production, but after Novell used it as cheap hardware to go with their software it became popular.
It is a good first network card to program because it follows a simple design (making it helpful for learning), they're probably dirt-cheap, and it is supported by most PC emulators. Both [[Bochs]] and [[QEMU]] provide ISA and PCI implementations.

Ne2000 is not technically a card, it is a standard that several implementors follow. The best available description of the initial standard was located [http://www.national.com/pf/DP/DP8390D.html DP8390D/NS32490D NIC Network Interface Controller] ([https://web.archive.org/web/20010612150713/http://www.national.com/ds/DP/DP8390D.pdf archive]) and was published by National Semiconductor.

== Quick Overview of the NIC design ==

The Ne2000 network card uses two ''ring buffers'' for packet handling. These are circular buffers made of 256-byte ''pages'' that the chip's [[DMA]] logic will use to store received packets or to get received packets.

Note that a packet will ''always'' start on a page boundary, thus there may be unused
bytes at the end of a page.

==== Ring Buffer ====

Two registers <tt>PSTART</tt> and <tt>PSTOP</tt> define a set of 256-byte pages in the ''buffer memory'' that will be used for the ring buffer. As soon as the DMA attempts to read/write to <tt>PSTOP</tt>, it will be sent back to <tt>PSTART</tt>

 PSTART                                                                       PSTOP
 ####+-8------+-9------+-a------+-b------+-c------+-d------+-e------+-f------+####
 ####| Packet 3 (cont) |########|########|Packet1#|   Packet  2#####|Packet 3|####
 ####+--------+--------+--------+--------+--------+--------+--------+--------+####
 
 An 8-page ring buffer with 3 packets and 2 free slots.

While receiving, the NIC has 2 additional registers that point to the first packet
that's still to be read and to the start of the currently written packet (named ''boundary pointer'' and ''current page'' respectively).

=== Register Pages ===

Programming registers of the NE2000 are collected in ''pages''. Page 0 contains most of the ''control and status'' registers while page 1 contains physical (PAR0..PAR5) and multicast addresses (MAR0..MAR7) to be checked by the card.

Note that the same register number could have a different meaning depending whether you ''read'' or ''write'' to it. For instance, register 0x0C on page 0 is the ''receive configuration register'' in ''write'' mode and "receive status register" in ''read'' mode. Most of the ''configuration'' registers can still be read on page 2 though. Each register is a single byte and the page is selected by highest 2 bits of the COMMAND register (which is available in all pages)

<pre>
Ne2K_registers (page=0, read) {
   COMMAND=0,          //!< the master command register
   CLDA0,              //!< Current Local DMA Address 0
   CLDA1,              //!< Current Local DMA Address 1
   BNRY,               //!< Boundary Pointer (for ringbuffer)
   TSR,                //!< Transmit Status Register
   NCR,                //!< collisions counter
   FIFO,               //!< (for what purpose ??)
   ISR,                //!< Interrupt Status Register
   CRDA0,              //!< Current Remote DMA Address 0
   CRDA1,              //!< Current Remote DMA Address 1
   RSR=0x0c,           //!< Receive Status Register
};

/*Registers that are the same in read & write are omitted.*/
Ne2K_registers (page=0, write) {
   PTART=1,            //!< page start (init only)
   PSTOP,              //!< page stop  (init only)
   TPSR=4,             //!< transmit page start address
   TBCR0,              //!< transmit byte count (low)
   TBCR1,              //!< transmit byte count (high)
   RSAR0=8,            //!< remote start address (lo)
   RSAR1,              //!< remote start address (hi)
   RBCR0,              //!< remote byte count (lo)
   RBCR1,              //!< remote byte count (hi)
   RCR,                //!< receive config register
   TCR,                //!< transmit config register
   DCR,                //!< data config register    (init)
   IMR,                //!< interrupt mask register (init)
};
</pre>

=== Initialization and MAC Address ===
This wasn't exactly obvious, but by looking at the ''ne2k-pci'' module from Linux I managed to figure out how to initilize the card and read its MAC address:

 nif->iobase = nif->pcidev->bar[0] & ~0x3;
 
 outb(nif->iobase + 0x1F, inb(nif->iobase + 0x1F));  // write the value of RESET into the RESET register
 while ((inb(nif->iobase + 0x07) & 0x80) == 0);      // wait for the RESET to complete
 outb(nif->iobase + 0x07, 0xFF);                     // mask interrupts
 
 uint8_t prom[32];
 outb(nif->iobase, (1 << 5) | 1);	// page 0, no DMA, stop
 outb(nif->iobase + 0x0E, 0x49);		// set word-wide access
 outb(nif->iobase + 0x0A, 0);		// clear the count regs
 outb(nif->iobase + 0x0B, 0);
 outb(nif->iobase + 0x0F, 0);		// mask completion IRQ
 outb(nif->iobase + 0x07, 0xFF);
 outb(nif->iobase + 0x0C, 0x20);		// set to monitor
 outb(nif->iobase + 0x0D, 0x02);		// and loopback mode.
 outb(nif->iobase + 0x0A, 32);		// reading 32 bytes
 outb(nif->iobase + 0x0B, 0);		// count high
 outb(nif->iobase + 0x08, 0);		// start DMA at 0
 outb(nif->iobase + 0x09, 0);		// start DMA high
 outb(nif->iobase, 0x0A);		// start the read
 
 int i;
 for (i=0; i<32; i++)
 {
   prom[i] = inb(nif->iobase + 0x10);
 };
 
 // program the PAR0..PAR5 registers to listen for packets to our MAC address!		
 for (i=0; i<6; i++)
 {
   writeRegister(nif, 1, 0x01+i, prom[i]);
 };

The first 6 bytes of "prom" extracted here are the MAC address.

=== Sending a Packet ===

The following sequence is the one observed by the ''ne2k-pci'' module in linux. Note that some odd cards needs a patch (read-before-write) that isn't covered here. The ''data configuration'' is initialized at 0x49 (word transfer, 8086 byte order, dual 16bit DMA, loopback disabled). Note that the weird driver doesn't seem to use interrupts for completion notification.

# <tt>COMMAND</tt> register set to "start" and "nodma" (0x22)
# <tt>RBCRx</tt> are loaded with the packet size
# "Remote DMA complete?" bit is cleared by writing a 1 in bit 6 of <tt>ISR</tt> (that's odd, but that's the way it works)
# <tt>RSARx</tt> are loaded with 0x00 (low) and target page number (high) respectively. At this stage, the chip is ready receiving packet data and storing it in the ring buffer for emission.
# <tt>COMMAND</tt> register set to "start" and "remote write DMA" (0x12)
# Packets data is now written to the "data port" (that is register 0x10) of the NIC in a loop (or using an "outsx" if available). The NIC will then update its remote DMA logic after each written 16-bit value/32-bit value and places bytes in the transmit ring buffer.
# Poll <tt>ISR</tt> register until bit 6 (Remote DMA completed) is set.

== ISA configuration information ==

=== Ne2000 Registers ===

The base register number can be anywhere from 0x280 to 0x380, as I've found, but I usually configure bochs to operate with port 0x300 as a base.

=== Ne2000 Interrupts ===

I have configured my ne2000 card in bochs to signal interrupts on [[IRQ]] 3.

=== Ne2000 Reset ===

Before transmitting data with the ne2000, it must be reset and data in the ring buffer cleared.
This can be done by writing out the contents of the reset register to the reset register.

==See Also==
===External Links===
* [https://web.archive.org/web/20010612150713/http://www.national.com/ds/DP/DP8390D.pdf DP8390D/NS32490D NIC Network Interface Controller (PDF)] datasheet for the 8390 chip on the NE2000
* https://bitsavers.org/components/national/_dataBooks/1988_National_Data_Communications_Local_Area_Networks_UARTs_Handbook.pdf, early description of NE1000(?) boards (including PROM and memory map), starting on page 124 ("DP839EB Network Evaluation Board - Application Note 479")
* [https://web.archive.org/web/20091223070134/http://www.national.com/pf/DP/DP8390D.html#Documents DP8390D Additional application notes] from NatSemi
* http://www.ethernut.de/pdf/8019asds.pdf, the RTL8019 is one of the PCI-based NE2K compliant cards.
* http://www.bcgreen.com/gnu-linux/ne2k-diag.c, a diagnostic tool that needs to be inspected to see if it helps in understanding the manuals
* http://www.cs.usfca.edu/~cruse/cs326/RTL8139_ProgrammersGuide.pdf, saved here for later use.
* https://github.com/torokernel/torokernel/blob/7d6df4c40fa4cc85febd5fd5799404592ffdff53/rtl/drivers/Ne2000.pas, example of a driver for Ne2000 in Freepascal.

[[Category:Network Hardware]]
[[Category:Standards]]
````

<a id="w001628"></a>

## W001628 — Talk:Ne2000

Bron-ID: W001628
Namespace: Talk (1)
Revisie: 2232 | Datum: 2006-12-15T09:15:37Z
Bron: https://wiki.osdev.org/Talk:Ne2000
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ANe2000&oldid=2232

````text
== The [[Ne2000]] Page might need work ==
Did I do alright? --''[[User:Brynet-Inc|Brynet-Inc]]''

: Maybe others kan add more info, but I think it looks good. However I can't tell much about the technical stuff, because I haven't programmed a Ne2000 yet. :-) I just linked some of the words in it and minor formatting changes. --[[User:Walling|Walling]] 02:56, 15 December 2006 (CST)

==Dead link==
 http://www.scyld.com/ne2k_pci.html is the website of the Linux PCI-based NE2000 driver, giving a list of compliant chips, horrors, and things alike...

The [http://www.scyld.com/ne2k_pci.html link] seems to be dead (redirects to beowulf.org).

archive.org has a copy here: http://web.archive.org/web/20051212214242/http://www.scyld.com/ne2k_pci.html

How do we want to handle this?
-[[User:Jhawthorn|Jhawthorn]] 03:08, 15 December 2006 (CST)

: Maybe you could just insert the link to the archive.org cached site? It's there. --[[User:Walling|Walling]] 03:15, 15 December 2006 (CST)
````

<a id="w001629"></a>

## W001629 — Talk:Loopback Device

Bron-ID: W001629
Namespace: Talk (1)
Revisie: 14896 | Datum: 2013-07-03T14:19:10Z
Bron: https://wiki.osdev.org/Talk:Loopback_Device
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ALoopback_Device&oldid=14896

````text
This article talks about creating, mounting, and formating images under Linux and *BSD. Being an OSDev wiki, however, I believe there should be theory behind loopback devices, discussion over the problems they cause (such as how in a microkernel a file operation on a loopback device may be cause a message to be sent through shell->vfs server->loopback server->vfs server->fat server), and how to implement a loopback device in an operating system.
--[[User:MessiahAndrw|MessiahAndrw]] 00:24, 15 December 2006 (CST)

Be my guest - [[User:Combuster|Combuster]] 05:32, 15 December 2006 (CST)

== Copy Stage files ==

 cp /lib/grub/i386-pc/stage[12] /mnt/myfloppy/grub

This folder no longer exists in modern distributions.

--[[User:Funyotros|Funyotros]] 14:23, 27 May 2013 (CDT)

== Article Name ==

The documentation that I've read is always careful to point out that a "loop device" is a different thing than a "loopback device" -- and that what this article is talking about is a loop device, not a loopback device. Should we fix this? [[User:Bewing|Bewing]] 22:25, 28 June 2013 (CDT)

Absolutely! possibly with a redirect from Loopback Device? --[[User:Aj|Aj]] 09:18, 3 July 2013 (CDT)
````

<a id="w001641"></a>

## W001641 — User:Jhawthorn

Bron-ID: W001641
Namespace: User (2)
Revisie: 2389 | Datum: 2006-12-16T23:53:57Z
Bron: https://wiki.osdev.org/User:Jhawthorn
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJhawthorn&oldid=2389

````text
__NOEDITSECTION__

<table align="right" cellpadding="4" cellspacing="0" style="margin-top:1em; margin-bottom:1em; background:#f9f9f9; border:1px #aaa solid; border-collapse:collapse; width: 30em; font-size: 90%; text-align: left;">
<tr><td colspan="2" style="text-align: center; font-size: 130%; font-weight: bold;">
John Hawthorn
</td></tr>
<tr>
<td>'''My Sandbox:'''</td><td>[[User:Jhawthorn/Sandbox]]</td>
</tr>
<tr>
<td>'''My Website:'''</td><td>http://www.johnhawthorn.com/</td>
</tr>
</table>









==My User Pages==
{{Special:Prefixindex/User:Jhawthorn}}
````

<a id="w001642"></a>

## W001642 — File:Osdev wallpaper 01.png

Bron-ID: W001642
Namespace: File (6)
Revisie: 2265 | Datum: 2006-12-16T15:29:40Z
Bron: https://wiki.osdev.org/File:Osdev_wallpaper_01.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AOsdev_wallpaper_01.png&oldid=2265

````text
[[Category:Wallpaper]]
````

<a id="w001643"></a>

## W001643 — Category:Wallpaper

Bron-ID: W001643
Namespace: Category (14)
Revisie: 2344 | Datum: 2006-12-16T21:00:32Z
Bron: https://wiki.osdev.org/Category:Wallpaper
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AWallpaper&oldid=2344

````text
Desktop wallpapers for the OSDev'er in you.

[[Category:Resources]]
````

<a id="w001644"></a>

## W001644 — File:Osdev wallpaper 02.png

Bron-ID: W001644
Namespace: File (6)
Revisie: 2267 | Datum: 2006-12-16T15:31:57Z
Bron: https://wiki.osdev.org/File:Osdev_wallpaper_02.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AOsdev_wallpaper_02.png&oldid=2267

````text
[[Category:Wallpaper]]
````

<a id="w001645"></a>

## W001645 — File:Osdev wallpaper 03.png

Bron-ID: W001645
Namespace: File (6)
Revisie: 2268 | Datum: 2006-12-16T15:32:35Z
Bron: https://wiki.osdev.org/File:Osdev_wallpaper_03.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AOsdev_wallpaper_03.png&oldid=2268

````text
[[Category:Wallpaper]]
````

<a id="w001646"></a>

## W001646 — File:Osdev wallpaper 04.png

Bron-ID: W001646
Namespace: File (6)
Revisie: 2269 | Datum: 2006-12-16T15:33:11Z
Bron: https://wiki.osdev.org/File:Osdev_wallpaper_04.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AOsdev_wallpaper_04.png&oldid=2269

````text
[[Category:Wallpaper]]
````

<a id="w001647"></a>

## W001647 — File:Osdev wallpaper 05.png

Bron-ID: W001647
Namespace: File (6)
Revisie: 2270 | Datum: 2006-12-16T15:33:56Z
Bron: https://wiki.osdev.org/File:Osdev_wallpaper_05.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AOsdev_wallpaper_05.png&oldid=2270

````text
[[Category:Wallpaper]]
````

<a id="w001648"></a>

## W001648 — File:Osdev wallpaper 06.png

Bron-ID: W001648
Namespace: File (6)
Revisie: 2271 | Datum: 2006-12-16T15:34:37Z
Bron: https://wiki.osdev.org/File:Osdev_wallpaper_06.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AOsdev_wallpaper_06.png&oldid=2271

````text
[[Category:Wallpaper]]
````

<a id="w001649"></a>

## W001649 — File:Osdev wallpaper 07.png

Bron-ID: W001649
Namespace: File (6)
Revisie: 2272 | Datum: 2006-12-16T15:35:14Z
Bron: https://wiki.osdev.org/File:Osdev_wallpaper_07.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AOsdev_wallpaper_07.png&oldid=2272

````text
[[Category:Wallpaper]]
````

<a id="w001650"></a>

## W001650 — File:Osdev wallpaper 08.png

Bron-ID: W001650
Namespace: File (6)
Revisie: 2273 | Datum: 2006-12-16T15:35:44Z
Bron: https://wiki.osdev.org/File:Osdev_wallpaper_08.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AOsdev_wallpaper_08.png&oldid=2273

````text
[[Category:Wallpaper]]
````

<a id="w001651"></a>

## W001651 — File:Osdev wallpaper 09.png

Bron-ID: W001651
Namespace: File (6)
Revisie: 2274 | Datum: 2006-12-16T15:36:44Z
Bron: https://wiki.osdev.org/File:Osdev_wallpaper_09.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AOsdev_wallpaper_09.png&oldid=2274

````text
[[Category:Wallpaper]]
````

<a id="w001652"></a>

## W001652 — Template:Box

Bron-ID: W001652
Namespace: Template (10)
Revisie: 12730 | Datum: 2012-02-19T20:30:37Z
Bron: https://wiki.osdev.org/Template:Box
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3ABox&oldid=12730

````text
<div style="margin: 0 0 10px 0; padding: 0 1em 0.7em 1em; background-color: #F5FAFF; color: #000000; border: 1px solid #CEDFF2; padding:0.6em 0.8em;">

<h2 style="margin:0;background-color:#CEDFF2;font-size:120%;font-weight:bold;border:1px solid #A3B0BF;text-align:left;color:#000;padding:0.2em 0.4em;">{{{subject}}}</h2>
{{{{{subject}}}}}
<div style="text-align: right; margin: 0; padding: 0;"><small>[[:Category:{{{subject}}}|more...]]</small></div>
</div>
````

<a id="w001653"></a>

## W001653 — Template:Help Us

Bron-ID: W001653
Namespace: Template (10)
Revisie: 29141 | Datum: 2024-06-22T23:00:45Z
Bron: https://wiki.osdev.org/Template:Help_Us
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AHelp_Us&oldid=29141

````text
<div style="margin: 0 0 8px 0; padding: 0.1em; background-color: #EFEFFF; color: #000000; border: 1px solid #9F9FFF; text-align: center;">
'''The OSDev Wiki welcomes [https://forum.osdev.org/viewtopic.php?t=677 authors]. See the [[Wish List]] for suggestions on where to start.'''
</div>
````

<a id="w001654"></a>

## W001654 — User:Chase/temp

Bron-ID: W001654
Namespace: User (2)
Revisie: 29137 | Datum: 2024-06-22T22:43:05Z
Bron: https://wiki.osdev.org/User:Chase/temp
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AChase%2Ftemp&oldid=29137

````text
{{Flex columns
|1 = (...first column contents...)
|2 = (...second column contents...)
}}

<div id="mainpage"></div> __NOTOC__ __NOEDITSECTION__

<!-- Welcome to OSDev.org box -->
{{Welcome}}

<!-- Conversion from OSFAQ wiki to MediaWiki box -->
{{Conversion}}


<!-- Table stuff -->
{| style="width: 100%; margin: 0; padding: 0; border: 0; border-collapse: collapse;"
| style="padding: 0; width: 25%; vertical-align: top;" |

<!-- First column -->
{{Box|subject= OS development}}
{{Box|subject= Resources }}

| style="padding: 0 0 0 10px; width: 25%; vertical-align: top;" |

<!-- Second column -->
{{Box|subject= Languages |more=yes}}
{{Box|subject= Tools |more=yes}}

| style="padding: 0 0 0 10px; width: 25%; vertical-align: top;" |

<!-- Third column -->
{{Box|subject= Hardware |more=yes}}
{{Box|subject= Troubleshooting |more=yes}}

| style="padding: 0 0 0 10px; width: 25%; vertical-align: top;" |

<!-- Forth column -->
{{Box|subject= OS theory }}

|}

[[Category:Wiki administration]]
````

<a id="w001655"></a>

## W001655 — User:Walling/Archive

Bron-ID: W001655
Namespace: User (2)
Revisie: 8157 | Datum: 2009-06-04T14:24:50Z
Bron: https://wiki.osdev.org/User:Walling/Archive
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AWalling%2FArchive&oldid=8157

````text
This is [[User:Walling|Walling's]] archive. His old user pages:

* [[/Sandbox]] - archived 2006-12-16
* [[/Sandbox2]] - archived 2006-12-16
* [[/UserPage1]] - archived 2009-06-04
````

<a id="w001656"></a>

## W001656 — User:Walling/Sandbox

Bron-ID: W001656
Namespace: User (2)
Revisie: 2311 | Datum: 2006-12-16T20:04:29Z
Bron: https://wiki.osdev.org/User:Walling/Sandbox
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AWalling%2FSandbox&oldid=2311

````text
'''Hello, World!'''
````

<a id="w001662"></a>

## W001662 — Serial Ports

Bron-ID: W001662
Namespace: Main (0)
Revisie: 30963 | Datum: 2026-09-22T18:13:59Z
Bron: https://wiki.osdev.org/Serial_Ports
Revisie-URL: https://wiki.osdev.org/index.php?title=Serial_Ports&oldid=30963

````text

Serial ports are a legacy communications port common on IBM-PC compatible computers. Use of serial ports for connecting peripherals has largely been deprecated in favor of [[USB]] and other modern peripheral interfaces, however it is still commonly used in certain industries for interfacing with industrial hardware such as CNC machines or commercial devices such as POS terminals. Historically it was common for many dial-up modems to be connected via a computer's serial port, and the design of the underlying [[UART]] hardware itself reflects this.

Serial ports are typically controlled by [[UART]] hardware. This is the hardware chip responsible for encoding and decoding the data sent over the serial interface. Modern serial ports typically implement the [[wikipedia:RS-232|RS-232]] standard, and can use a variety of different connector interfaces. The DE-9 interface is the one most commonly used connector for serial ports in modern systems.

Serial ports are of particular interest to operating-system developers since they are much easier to implement drivers for than USB, and are still commonly found in many x86 systems. It is common for operating-system developers to use a system's serial ports for debugging purposes, since they do not require sophisticated hardware setups and are useful for transmitting information in the early stages of an operating-system's initialization. Many emulators such as [[QEMU]] and [[Bochs]] allow the redirection of serial output to either stdio or a file on the host computer.

== Wires, Pins, Connectors and the like ==

The Wikipedia page on [[wikipedia:Serial_port|Serial ports]] has a lot of information, and it is summarised here. The serial interface is very simple. There are actually two kinds of serial port: 25-pin and 9-pin. 25-pin ports are not any better, they just have more pins (most unused) and are bigger. 9-pin is smaller and is used more often though in the past the 25-pin ones were used more often. The 9-pin ones are called DE-9 (or more commonly, DB-9 even though DE-9 is its technical name) and the 25-pin ones are called DB-25. They plug in to your computer using a female plug (unless your computer is odd and has a female port, in which case your cable will need a male plug). [[wikipedia:D-subminiature|This Wikipedia page]] has more information on the plug used. 

Both have the same basic types of pins. A DB-25 has most of the pins as ground pins or simply unconnected, whereas a DE-9 has only one ground pin. There is a transmitting pin (for sending information away) and a receiving pin (for getting information). Most serial ports run in a duplex mode--that is, they can send and receive simultaneously. There are a few other pins, used for hardware handshaking. In the past, there was no duplex mode, so if a computer wanted to send something it had to tell the other device or computer that it was about to transmit, using one of the hardware handshaking pins. The other device would then use another handshaking pin to tell it to send whatever it wanted to send. Today there is duplex mode, but the handshaking pins are still used.

If you want to connect two computers, you need two things in your cable:

# The cable needs to have two female plugs so it can plug into both computers.
# The cable needs to have its transmit-receive wires and it's handshaking wires switched. This can be done in the cable itself, or as an extension called a [[Null Modem]]

For serial devices, you don't need to setup the cable this way. The receiving end of the device has the wires switched and it has a female port, which means you can plug a male plug into it.

==Why Use a Serial Port?==
During the early stages of kernel development, you might wonder why you would bother writing a serial driver. There are several reasons why you might:
;[[GDB]] debugging
:You can use the serial port to connect to a host computer, and use the GDB debugger to debug your operating system. This involves writing a stub for GDB within your OS.
;Headless console
:You can operate the computer without a monitor, keyboard or mouse and instead use the serial port as a console using a protocol such as TTY or VT100.
;External logging
:When the system itself is in danger of potentially crashing at times, it's nice to get debugging outputs safe to another computer before the test system triple-faults.
;Networking and File transfers
:Serial ports are useful for transferring information between systems when other more traditional methods are unavailable.

==Programming the Serial Communications Port==
If you want to use the serial port for communications, you first have to initialize it. You tell it how fast your connection speed between the other computer or device will be (this is called the baud rate)--you must have the same speed as the other device or computer is setup to use, or you will have problems. It is probably safer to use the slower speeds unless you need the faster speeds for some reason, for example if you are playing a multi-player game over a serial connection. You also need to setup the parity type and the number of bits in a character. Once again, your computer must be setup with the same values for these things as the other computer or device has, or communication will not work.

Once you have setup these things, you still need to setup the interrupt handlers. You can poll the port to see if any new things have arrived, or if it's time to send another character, but this slows things down and will not work very well in most real-time applications or multi-threaded environments. In the case of a game, this is not a good idea at all.

You use IRQ #4 for COM ports 1 or 3, and IRQ #3 for COM ports 2 or 4 (you can tell which port sent the interrupt when you receive the interrupt). The IRQ handlers check if you are receiving something, and if so they receive the character and handle it somehow, such as placing it into a buffer. They also check if the other side is ready to receive something from you, and if you have something to send, it is sent.

===Port Addresses===
The addresses for COM ports can vary depending on how they are connected to the machine and how the BIOS is configured.  Some BIOS configuration utilities allow you to see and set what these are, so if you in doubt for a test machine, this might be a good place to look to get you started.

For the most part, the first two COM ports will be at the addresses specified, the addresses for further COM ports are less reliable.
{| class="wikitable"
! COM Port
! IO Port
|-
| COM1 || 0x3F8
|-
| COM2 || 0x2F8
|-
| COM3 || 0x3E8
|-
| COM4 || 0x2E8
|-
| COM5 || 0x5F8
|-
| COM6 || 0x4F8
|-
| COM7 || 0x5E8
|-
| COM8 || 0x4E8
|}

You might be able to find the IO port addresses of the COM ports in the [[Memory Map (x86)#BIOS_Data_Area_.28BDA.29|BIOS Data Area]]; however be warned that this won't work on modern/UEFI systems. The BDA can tell you about serial ports that only exist in the chipset (and lack any kind of connector that anything can be plugged into), won't tell you about any additional serial ports (e.g. on expansion cards, etc) that firmware doesn't/can't know about, and will make your OS susceptible to "BIOS quirks/bugs". Since the serial ports have relatively standard IO ports, it's far more effective to use manual probing techniques instead; specifically, see if the scratch pad register can store a value, then try the loopback test (that you should use to determine if the serial port is faulty anyway).

Once you have the base address of your COM port, you add an offset value to get to one of the data registers.  One of the registers hold what is termed the DLAB or Divisor Latch Access Bit.  When this bit is set, offsets 0 and 1 are mapped to the low and high bytes of the Divisor register for setting the baud rate of the port.  When this bit is clear, offsets 0 and 1 are mapped to their normal registers.  The DLAB bit only affects port offsets 0 and 1, the other offsets ignore this setting.

{| class="wikitable"
! IO Port Offset
! Setting of DLAB
! I/O Access
! Register mapped to this port
|-
| +0 || 0 || Read || Receive buffer.  
|-
| +0 || 0 || Write || Transmit buffer.
|-
| +1 || 0 || Read/Write || Interrupt Enable Register.
|-
| +0 || 1 || Read/Write || With DLAB set to 1, this is the least significant byte of the divisor value for setting the baud rate.
|-
| +1 || 1 || Read/Write || With DLAB set to 1, this is the most significant byte of the divisor value.
|-
| +2 || - || Read || Interrupt Identification
|-
| +2 || - || Write || FIFO control registers
|-
| +3 || - || Read/Write || Line Control Register.  The most significant bit of this register is the DLAB.
|-
| +4 || - || Read/Write || Modem Control Register.
|-
| +5 || - || Read || Line Status Register.
|-
| +6 || - || Read || Modem Status Register.
|-
| +7 || - || Read/Write || Scratch Register.
|}

===Line Protocol===
The serial data transmitted across the wire can have a number of different parameters set.  As a rule, the sending device and the receiving device require the same protocol parameter values written to each serial controller in order for communication to be successful.

These days you could consider 8N1 (8 bits, no parity, one stop bit) pretty much the default.

====Baud Rate====
The serial controller (UART) has an internal clock which runs at 115200 ticks per second and a clock divisor which is used to control the baud rate.  This is exactly the same type of system used by the Programmable Interrupt Timer (PIT).

In order to set the speed of the port, calculate the divisor required for the given baud rate and program that in to the divisor register.  For example, a divisor of 1 will give 115200 baud, a divisor of 2 will give 57600 baud, 3 will give 38400 baud, etc.

Do not be tempted to use a divisor of 0 to try to get an infinite baud rate, it won't work.  Most serial controllers will generate a unspecified and unpredictable baud rate (and anyway infinite baud would mean infinite transmission errors as they are proportional.)

To set the divisor to the controller:
# Set the most significant bit of the Line Control Register.  This is the DLAB bit, and allows access to the divisor registers.
# Send the least significant byte of the divisor value to [PORT + 0].
# Send the most significant byte of the divisor value to [PORT + 1].
# Clear the most significant bit of the Line Control Register.

====Line Control Register====
The Line Control register sets the general connection parameters.

{| class="wikitable" 
|-
! Bit 7
! Bit 6
! Bits 5-3
! Bit 2
! Bits 1-0
|-
| Divisor Latch Access Bit
| Break Enable Bit
| Parity Bits
| Stop Bits
| Data Bits
|}

====Data Bits====
The number of bits in a character is variable. Having fewer bits is, of course, faster, but they store less information. If you are only sending ASCII text, you probably only need 7 bits.

Set this value by writing to the two least significant bits of the Line Control Register [PORT + 3].
{| class="wikitable"
! Bit 1
! Bit 0
! Character Length (bits)
|-
| 0 || 0 || 5
|-
| 0 || 1 || 6
|-
| 1 || 0 || 7
|-
| 1 || 1 || 8
|}

====Stop Bits====
The serial controller can be configured to send a number of bits after each character of data.  These reliable bits can be used to by the controller to verify that the sending and receiving devices are in phase.

If the character length is specifically 5 bits, the stop bits can only be set to 1 or 1.5.  For other character lengths, the stop bits can only be set to 1 or 2.

To set the number of stop bits, set bit 2 of the Line Control Register [PORT + 3].

{| class="wikitable"
! Bit 2
! Stop bits
|-
| 0 || 1
|-
| 1 || 1.5 / 2 (depending on character length)
|}

====Parity Bits====
The controller can be made to add or expect a parity bit at the end of each character of data transmitted.  With this parity bit, if a single bit of data is inverted by interference, a parity error can be raised.  The parity type can be NONE, EVEN, ODD, MARK or SPACE.

If parity is set to NONE, no parity bit will be added and none will be expected.  If one is sent by the transmitter and not expected by the receiver, it will likely cause an error.

If the parity is MARK or SPACE, the parity bit will be expected to be always set to 1 or 0 respectively.

If the parity is set to EVEN or ODD, the controller calculates the accuracy of the parity by adding together the values of all the data bits and the parity bit.  If the port is set to have EVEN parity, the result must be even.  If it is set to have ODD parity, the result must be odd.

To set the port parity, set bits 3, 4 and 5 of the Line Control Register [PORT + 3].
{| class="wikitable"
! Bit 5
! Bit 4
! Bit 3
! Parity
|-
| - || - || 0 || NONE
|-
| 0 || 0 || 1 || ODD
|-
| 0 || 1 || 1 || EVEN
|-
| 1 || 0 || 1 || MARK
|-
| 1 || 1 || 1 || SPACE
|}

===Interrupt enable register===
To communicate with a serial port in interrupt mode, the interrupt-enable-register (see table above) must be set correctly. To determine which interrupts should be enabled, a value with the following bits (0 = disabled, 1 = enabled) must be written to the interrupt-enable-register:
{| class="wikitable"
! Bit 7-4
! Bit 3
! Bit 2
! Bit 1
! Bit 0
|-
| Reserved
| Modem Status
| Receiver Line Status
| Transmitter Holding Register Empty
| Received Data Available
|}

===First In First Out Control Register===
The First In / First Out Control Register (FCR) is for controlling the FIFO buffers. Access this register by writing to port offset +2.
{| class="wikitable" 
|-
! Bits 7-6
! Bits 5-4
! Bit 3
! Bit 2
! Bit 1
! Bit 0
|-
| Interrupt Trigger Level
| Reserved
| DMA Mode Select
| Clear Transmit FIFO
| Clear Receive FIFO
| Enable FIFO's
|}

====Clear Transmit FIFO and Clear Receive FIFO====
Bit 2 being set clears the Transmit FIFO buffer while Bit 1 being set clears the Receive FIFO buffer. Both bits will set themselves back to 0 after they are done being cleared.

====Interrupt Trigger Level====
The Interrupt Trigger Level is used to configure how much data must be received in the FIFO Receive buffer before triggering a Received Data Available Interrupt.
{| class="wikitable"
! Bit 7
! Bit 6
! Trigger Level
|-
| 0 || 0 || 1 Byte
|-
| 0 || 1 || 4 Bytes
|-
| 1 || 0 || 8 Bytes
|-
| 1 || 1 || 14 Bytes
|}

===Interrupt Identification Register===
The Interrupt Identification Register (IIR) is for identifying pending interrupts. Access this register by reading from port offset +2.
{| class="wikitable" 
|-
! Bits 7-6
! Bits 5-4
! Bit 3
! Bit 2-1
! Bit 0
|-
| FIFO Buffer State
| Reserved
| Timeout Interrupt Pending (UART 16550) or Reserved
| Interrupt State
| Interrupt Pending if 0
|}

====Interrupt State====
After Interrupt Pending is set, the Interrupt State shows the interrupt that has occurred. They have varying levels of priority, with high-value interrupts handled first, and low-value interrupts being handled last.
{| class="wikitable"
! Bit 2
! Bit 1
! Interrupt
! Priority
|-
| 0 || 0 || Modem Status || 4 (Lowest)
|-
| 0 || 1 || Transmitter Holding Register Empty || 3
|-
| 1 || 0 || Received Data Available || 2
|-
| 1 || 1 || Receiver Line Status || 1 (Highest)
|}

====FIFO Buffer State====
{| class="wikitable"
! Bit 7
! Bit 6
! State
|-
| 0 || 0 || No FIFO
|-
| 0 || 1 || FIFO Enabled but Unusable
|-
| 1 || 0 || FIFO Enabled
|}

===Modem Control Register===
The Modem Control Register is one half of the hardware handshaking registers. 
While most serial devices no longer use hardware handshaking, The lines are still included in all 16550 compatible UARTS. 
These can be used as general purpose output ports, or to actually perform handshaking. 
By writing to the Modem Control Register, it will set those lines active.
{| class="wikitable"
! Bit
! Name
! Meaning
|-
| 0 || Data Terminal Ready (DTR) || Controls the Data Terminal Ready Pin
|-
| 1 || Request to Send (RTS) || Controls the Request to Send Pin
|-
| 2 || Out 1 || Controls a hardware pin (OUT1) which is unused in PC implementations
|-
| 3 || Out 2 || Controls a hardware pin (OUT2) which is used to enable the IRQ in PC implementations
|-
| 4 || Loop || Provides a local loopback feature for diagnostic testing of the UART
|-
| 5 || 0 || Unused
|-
| 6 || 0 || Unused
|-
| 7 || 0 || Unused
|}

Most PC serial ports use OUT2 to control a circuit that disconnects (tristates) the IRQ line. This makes it possible for multiple serial ports to share a single IRQ line, as long as only one port is enabled at a time.
Loopback mode is a diagnostic feature. When bit 4 is set to logic 1,
the following occur the transmitter Serial Output (SOUT) is set to the Marking (logic 1) state; the receiver Serial
Input (SIN) is disconnected; the output of the Transmitter Shift Register is ‘‘looped back’’ into the Receiver Shift
Register input; the four MODEM Control inputs (DSR, CTS, RI, and DCD) are disconnected; and the four
MODEM Control outputs (DTR, RTS, OUT 1, and OUT 2) are internally connected to the four MODEM Control
inputs, and the MODEM Control output pins are forced to their inactive state (high). In the loopback mode, data
that is transmitted is immediately received. This feature allows the processor to verify the transmit-and received-
data paths of the UART. In the loopback mode, the receiver and transmitter interrupts are fully operational. Their sources are external to
the part. The MODEM Control Interrupts are also operational, but the interrupts’ sources are now the lower four
bits of the MODEM Control Register instead of the four MODEM Control inputs. The interrupts are still controlled
by the Interrupt Enable Register.

===Line Status Register===
The line status register is useful to check for errors and enable polling.
{| class="wikitable"
! Bit
! Name
! Meaning
|-
| 0 || Data ready (DR) || Set if there is data that can be read
|-
| 1 || Overrun error (OE) || Set if there has been data lost
|-
| 2 || Parity error (PE) || Set if there was an error in the transmission as detected by parity
|-
| 3 || Framing error (FE) || Set if a stop bit was missing
|-
| 4 || Break indicator (BI) || Set if there is a break in data input
|-
| 5 || Transmitter holding register empty (THRE) || Set if the transmission buffer is empty (i.e. data can be sent)
|-
| 6 || Transmitter empty (TEMT) || Set if the transmitter is not doing anything
|-
| 7 || Impending Error || Set if there is an error with a word in the input buffer
|}

===Modem Status Register===
This register provides the current state of the control lines from a peripheral device. 
In addition to this current-state information, four bits of the MODEM Status Register provide change information.
These bits are set to a logic 1 whenever a control input from the MODEM changes state. They are reset to logic
0 whenever the CPU reads the MODEM Status Register
{| class="wikitable"
! Bit
! Name
! Meaning
|-
| 0 || Delta Clear to Send (DCTS) || Indicates that CTS input has changed state since the last time it was read
|-
| 1 || Delta Data Set Ready (DDSR) || Indicates that DSR input has changed state since the last time it was read
|-
| 2 || Trailing Edge of Ring Indicator (TERI) || Indicates that RI input to the chip has changed from a low to a high state
|-
| 3 || Delta Data Carrier Detect (DDCD)  || Indicates that DCD input has changed state since the last time it ware read
|-
| 4 || Clear to Send (CTS) || Inverted CTS Signal
|-
| 5 || Data Set Ready (DSR) || Inverted DSR Signal
|-
| 6 || Ring Indicator (RI) || Inverted RI Signal
|-
| 7 || Data Carrier Detect (DCD) || Inverted DCD Signal
|}

If Bit 4 of the MCR (LOOP bit) is set, the upper 4 bits will mirror the 4 status output lines set in the Modem Control Register.

===Terminals===
:''Main article: [[Terminals]]

Once you can send and receive bytes with confidence, you probably want to connect the serial port to a terminal (or more likely a terminal emulator these days). Those send specific byte sequences when a key is pressed, and can interpret codes to move the cursor on the screen and change color for example.


==Example Code==
===Initialization===
<syntaxhighlight lang="C">
#define PORT 0x3f8          // COM1

static int init_serial() {
   outb(PORT + 1, 0x00);    // Disable all interrupts
   outb(PORT + 3, 0x80);    // Enable DLAB (set baud rate divisor)
   outb(PORT + 0, 0x03);    // Set divisor to 3 (lo byte) 38400 baud
   outb(PORT + 1, 0x00);    //                  (hi byte)
   outb(PORT + 3, 0x03);    // 8 bits, no parity, one stop bit
   outb(PORT + 2, 0xC7);    // Enable FIFO, clear them, with 14-byte threshold
   outb(PORT + 4, 0x0B);    // IRQs enabled, RTS/DSR set
   outb(PORT + 4, 0x1E);    // Set in loopback mode, test the serial chip
   outb(PORT + 0, 0xAE);    // Test serial chip (send byte 0xAE and check if serial returns same byte)

   // Check if serial is faulty (i.e: not same byte as sent)
   if(inb(PORT + 0) != 0xAE) {
      return 1;
   }

   // If serial is not faulty set it in normal operation mode
   // (not-loopback with IRQs enabled and OUT#1 and OUT#2 bits enabled)
   outb(PORT + 4, 0x0F);
   return 0;
}
</syntaxhighlight>

Notice that the initialization code above writes to [PORT + 1] twice with different values.  This is once to write to the Divisor register along with [PORT + 0] and once to write to the Interrupt register as detailed in the previous section.  The second write to the Line Control register [PORT + 3] clears the DLAB again as well as setting various other bits.

===Receiving data===
<syntaxhighlight lang="C">
int serial_received() {
   return inb(PORT + 5) & 1;
}

char read_serial() {
   while (serial_received() == 0);

   return inb(PORT);
}
</syntaxhighlight>

===Sending data===

<syntaxhighlight lang="C">
int is_transmit_empty() {
   return inb(PORT + 5) & 0x20;
}

void write_serial(char a) {
   while (is_transmit_empty() == 0);

   outb(PORT,a);
}
</syntaxhighlight>

==Glossary==

;Baud Rate
:The speed at which the serial line switches between it's two states. This is not equivalent to bps, due to the fact there are start and stop bits. On an 8/N/1 line, 10 baud = 1 byte. Modems are more complex than plain serial lines due to having multiple waveforms, but for the purposes of OSDev this is irrelevant.
:The fastest baud rate a serial port can reliably run at is generally 115200 baud.
;Baud Rate Divisor
:The value the is used by the UART to divide its internal clock by in order to get the actual intended baud rate.
;Stop Bits
:The NULL bit(s) sent between each character to synchronize the transmitter and the receiver.
;UART
:For Universal Asynchronous Receiver/Transceiver: the chip that picks a byte a send it bit per bit on the serial line and vice versa.

== See Also ==
=== External Links ===
* [https://web.archive.org/web/20120524222409/http://www.nondot.org/sabre/os/articles/CommunicationDevices/ OSRC's "communication" section]
* [http://www.sci.muni.cz/docs/pc/serport.txt A summary of the PC serial port (with programming info).], [http://www.webcitation.org/5ugQv5JOw An archive of that document]
* [http://en.wikibooks.org/wiki/Serial_Programming A Wikibook about programming the serial port]
* [https://docs.freebsd.org/en/articles/serial-uart/index.html Good overview of PC's UART in FreeBSD docs]

[[Category:Network Hardware]]
[[Category:Common Devices]]
[[de:Serielle_Schnittstelle]]
````

<a id="w001663"></a>

## W001663 — Category:Theory

Bron-ID: W001663
Namespace: Category (14)
Revisie: 2324 | Datum: 2006-12-16T20:40:16Z
Bron: https://wiki.osdev.org/Category:Theory
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ATheory&oldid=2324

````text
Articles regarding general theory, like math. Also see the category [[:Category:OS theory|OS theory]].

[[Category:OS theory]]
````

<a id="w001664"></a>

## W001664 — Category:OS theory

Bron-ID: W001664
Namespace: Category (14)
Revisie: 2327 | Datum: 2006-12-16T20:44:47Z
Bron: https://wiki.osdev.org/Category:OS_theory
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AOS_theory&oldid=2327

````text
Articles about OS theory.

[[Category:Main Page Topics]]
````

<a id="w001665"></a>

## W001665 — Category:Main Page Topics

Bron-ID: W001665
Namespace: Category (14)
Revisie: 2348 | Datum: 2006-12-16T21:05:37Z
Bron: https://wiki.osdev.org/Category:Main_Page_Topics
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AMain_Page_Topics&oldid=2348

````text
Listing categories for the topics on the [[Main Page]].

[[Category:Categories]]
````

<a id="w001668"></a>

## W001668 — Category:Resources

Bron-ID: W001668
Namespace: Category (14)
Revisie: 2338 | Datum: 2006-12-16T20:55:23Z
Bron: https://wiki.osdev.org/Category:Resources
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AResources&oldid=2338

````text
Different resources on OS development.

[[Category:Main Page Topics]]
````

<a id="w001671"></a>

## W001671 — Category:Categories

Bron-ID: W001671
Namespace: Category (14)
Revisie: 12679 | Datum: 2012-02-19T18:57:51Z
Bron: https://wiki.osdev.org/Category:Categories
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ACategories&oldid=12679

````text
The highest level of the OSDev category system. Do not add articles and other categories unless you are sure it must be added here.
[[Category:Categories]]
````

<a id="w001672"></a>

## W001672 — Category:Standards

Bron-ID: W001672
Namespace: Category (14)
Revisie: 2363 | Datum: 2006-12-16T21:15:31Z
Bron: https://wiki.osdev.org/Category:Standards
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AStandards&oldid=2363

````text
Listing OS relevant standards.

[[Category:Resources]]
````

<a id="w001673"></a>

## W001673 — Category:Troubleshooting

Bron-ID: W001673
Namespace: Category (14)
Revisie: 5587 | Datum: 2008-06-04T19:32:05Z
Bron: https://wiki.osdev.org/Category:Troubleshooting
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ATroubleshooting&oldid=5587

````text
When you have trouble developing your OS.

[[Category:OS Development]]
````

<a id="w001675"></a>

## W001675 — Category:Video

Bron-ID: W001675
Namespace: Category (14)
Revisie: 4368 | Datum: 2007-08-17T10:36:54Z
Bron: https://wiki.osdev.org/Category:Video
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AVideo&oldid=4368

````text
[[Category:Hardware]]
This contains all the pages that have got something to do with graphics.
````

<a id="w001676"></a>

## W001676 — Category:Memory management

Bron-ID: W001676
Namespace: Category (14)
Revisie: 3713 | Datum: 2007-04-25T22:07:23Z
Bron: https://wiki.osdev.org/Category:Memory_management
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AMemory_management&oldid=3713

````text
This category covers the theory and implementation topics that are a part of memory management design in operating system [[Kernels|kernels]].
{{main|Memory management}}

[[Category:OS theory]]
````

<a id="w001677"></a>

## W001677 — Category:VGA

Bron-ID: W001677
Namespace: Category (14)
Revisie: 2612 | Datum: 2006-12-29T23:47:23Z
Bron: https://wiki.osdev.org/Category:VGA
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AVGA&oldid=2612

````text
This category lists all topics describing the IBM VGA

[[Category:Video]]
````

<a id="w001679"></a>

## W001679 — User talk:Jhawthorn

Bron-ID: W001679
Namespace: User talk (3)
Revisie: 3993 | Datum: 2007-05-28T00:26:11Z
Bron: https://wiki.osdev.org/User_talk:Jhawthorn
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AJhawthorn&oldid=3993

````text
== Listing pages ==

Very nice! I didn't know you could include the special prefixindex page as a template, listing all user pages. Thanks. --[[User:Walling|Walling]] 18:05, 16 December 2006 (CST)
:Glad it helped -[[User:Jhawthorn|Jhawthorn]] 00:49, 17 December 2006 (CST)

==Notes==
Note to self: Fix hideous babystep tutorials.

== Misc ==
:: > ''(Original Message: [[User_talk:Kmcguire]])''
:: > ''I like using a template for the [[Projects]] page. Good work. -[[User:Jhawthorn|Jhawthorn]] 18:42, 27 May 2007 (CDT)''
[[Image:Sized_Portion_Scheme.png|Size_Portion_Scheme.png|frame|right]]
: '''T'''hanks, I am glad I got it accomplished and working. And to return the good remarks I do like the images you have been creating over time. I especially like the new one such as to the (''right''). The fading on the ends is very creative and makes a very professional look.
: '''O'''n the same note too if for some reason it does not work very well you guys can always rollback the changes. Due to murphy's law some people might find it harder to edit the page using those templates than with out.
: --[[User:Kmcguire|Kevin]] 19:26, 27 May 2007 (CDT)
````

<a id="w001680"></a>

## W001680 — Languages

Bron-ID: W001680
Namespace: Main (0)
Revisie: 27206 | Datum: 2022-07-01T04:41:39Z
Bron: https://wiki.osdev.org/Languages
Revisie-URL: https://wiki.osdev.org/index.php?title=Languages&oldid=27206

````text
There are [[:Category:Languages|many]] '''programming languages''', some more suited for OS development and kernel writing than others. This page will discuss it in depth.

== History ==

Early operating systems were written entirely in the [[Assembly]] language dialect of their respective CPU, and it remains an option for current developers willing to put in the time and effort to use it. Even when an OS is written primarily in a high-level language, there are still parts that can only be done in assembly. A significant sub-set of OS devs choose to work exclusively in assembly, and at least some work in machine language directly (though this is rare today).

Many high-level languages have been used for OS development in the past, including [[C]], Forth, Lisp, [[CSharp|C#]], [[C++]], Modula-2, [[Ada]], Bliss, Smalltalk, and PL/1. However, not all languages as are suited for OS development, and in many languages other than C, a fair amount of Assembly development is required in order to provide the appropriate runtime environment supporting the language's abstractions. Languages such as C, Modula-2, Ada, Bliss, PL/M, and XPL have all been designed specifically for the purpose of low-level systems programming, either in OS dev or embedded systems, while languages such as Forth incorporate the necessary low-level features even when they weren't intended specifically for this purpose.

== Warning ==

Not all languages are suited for low-level system programming. Some have no suitable low-level development tools available, or require specific runtime support -- problems C does not suffer from. Also, the vast majority of OS related resources (like tutorials, and how-to examples, including this wiki) assume C as the primary development language, so an OS developer should at least be able to read C code.

Using a language other than C entails a good deal of extra effort. But there are [[Alta Lang |some developers]] who are willing to put in that effort in order to suit the development of their OSes to their ways of thinking (for example: [https://en.wikipedia.org/wiki/Programming_paradigm programming paradigms]).

On the other hand, attempting to write an OS in interpreted or bytecode languages like Perl or Java is prohibitively challenging, and therefore less likely to lead to a success story. There are research projects about it, of course, but nothing that has completely changed the way we write kernels so far. And trying to write it in HTML would just be proof that you have much to learn.

So, you're warned. And now you should make a choice, hopefully, the informed choice.

== Can I use language XYZ? ==

; If you'd like to know if your favourite language is suited for OSDeving, just consider the most important principle - there should be a way of doing low level things in your language.

Also before you start it worth to answer the following questions: 
* Can you cope with datastructures having a specific bits & bytes arrangement (mandatory for e.g. MMU structures and alike things)? Or do you have corresponding tools?
* Can you take control of memory allocation/freeing? Or can you at least subdivide a large chunk of memory in smaller chunks that other functions can use transparently (necessary for any sort of memory management)? And the important consequence here is - do you have some kind of support of your personal memory manager in your language?
* Are you able to build a self-sufficient run-time library to support language features you'll need?
* Can you easily interface XYZ with some assembly code (yes, you'll have ''some'', at least in the run-time library you'll have to write)?
* If XYZ fits the other points and is an interpreted language, can you invoke code coming from raw data bytes with XYZ, i.e. jump at a specific address and continue execution there (this will be mandatory for loading and running programs)?

If any of those question turns out to "whoops, no, I cannot do that with language XYZ", then chances are that XYZ will be of no help for OS development. This particular problem has often been circumvented by modifying the language and writing new compilers.

== Can't I write a compiler for XYZ? ==

A compiler is one of the few things in software that are of a similar complexity to operating systems. As you are already planning to write an OS, deciding to do a compiler also is like finding new ways to forge metals in order to build a better car. Still, there are [[Alta Lang|some OSDevers]] who think themselves up to the challenge. The canonical starting point is the "Dragon Book" (''Compilers - Principles, Techniques, and Tools'' located on the [[Books]] page).

This does not mean you can afford to ignore C and Assembly, however; indeed, Assembly can be much more important to the code generation step of compilation than it is to OS development (unless you are eschewing high-level languages entirely), and for compiler designers a solid grasp of the target language is a must. Also, as has already been stated, most of the information about OS dev is oriented towards C developers, and a solid ability to at least read C is necessary unless you intend to work wholly alone.

Also, consider this: what language will you write your compiler in? Assembly is clunky, so your choices are C, or your favorite language, XYZ. If you use the latter, how will bootstrap yourself? If you use the former, you'll need to port libc and GCC -- you ''do'' want to be self-hosting... right?

== But I heard of an OS written in language XYZ, isn't it interpreted? ==

This is a red herring.  There is no such thing as an "interpreted language".  Any language can be implemented using either an interpreter or a compiler; and even in an OS project, there are forms of 'interpretation' which can nevertheless by applied to system operations.

You may from time to time hear of operating systems written in languages which are usually interpreted, or which used an interpreter of some sort: JavaOS, [http://en.wikipedia.org/wiki/Genera_%28operating_system%29 Genera] (the Symbolics Lisp Machine OS), Pilot-OS (the system for the [https://en.wikipedia.org/wiki/Xerox_Star Xerox Star] workstation, written mostly in the Mesa language), UCSD Pascal, the various Forth systems, etc. Most of these fall into one of three categories : 
* The operating system runs in a low-level interpreter, written in Assembly or some systems language like C, which is what actually interacts with the hardware. In effect, the 'operating system' is just an application running on top of another, lower-level OS. Pilot-OS, UCSD Pascal, and some Java OSes work like this, though they also have some modules which are compiled to native code as well (see below).

* All or part of the code has been compiled to native code. This may involve using a sub-set of the language with reduced runtime requirements (e.g., [http://www.citeulike.org/user/hamish/article/201596 Pre-Scheme], or [http://www.squeak.org/features/vm.html Slang] - while they have not been used for OS development to date, they do demonstrate this sort of low-level implementation language which can be used this way).
** '''Forth'''-based operating systems are a special case of this. While Forth is usually described as an interpreted language, the [http://en.wikipedia.org/wiki/Threaded_code threaded-code interpreters] many FORTH systems use work differently from most other interpreters; in effect, the interpreter walks through the various Forth 'words' that the code is comprised of until it reaches the low-level words that are implemented in assembly or compiled code, which is what actually gets executed. Furthermore, Forth systems incorporate a special sort of assembler, which produces code specifically meant to be used by the interpreter; also, commonly used 'words' can be compiled into native code as needed. Finally, many embedded Forth systems use special-purpose hardware (see below) to support the language.
** Most '''Lisp''' systems freely mix interpreted and compiled code, and in nearly all 'serious' Lisp systems since the late 1970s, the Listener REPL (the 'command line' for the Lisp system) is not an interpreter, but rather a compile-and-go system - each piece of code is at least partially compiled on the fly. While no significant Lisp OS has been made without also using language-specific hardware support (see below), if one were, it surely would not be a purely interpreted system. 
* The system ran on specialized hardware and microcode, which acted as hardwired 'interpreter' for it's primary language, or for the portable bytecode which it normally used. This type of system includes the SOAR (Smalltalk On A RISC), the Recursiv System, The Lillith Modula-2 System, and the Burroughs 6500 (a mainframe designed for running Algol-60 in the 1960s). The system programming techniques for these cannot work on stock hardware. For example: 
** The '''[http://en.wikipedia.org/wiki/MIT_CADR MIT CADR] Lisp machine architecture''' had an extensive instruction set with hardware support for certain high-level operations such as type-tag checking and GC.  It had a [http://en.wikipedia.org/wiki/Tagged_architecture tagged architecture] meaning that a portion of the 36-bit addressing word was designated for type information.  Typically these machines had a variety of compilers including one for the system language Lisp which was capable of taking advantage of the additional instruction set.
** The '''Rekursiv Single-Board Computer''' had hardware support for a writable instruction set (that is, you could dynamically add microcode instructions) and associative memory dispatch tables for supporting object-oriented programming.
[[Category:Languages]]
[[Category:OS Development]]
[[Category:FAQ]]
````

<a id="w001682"></a>

## W001682 — Bootloader

Bron-ID: W001682
Namespace: Main (0)
Revisie: 30966 | Datum: 2026-09-22T22:19:12Z
Bron: https://wiki.osdev.org/Bootloader
Revisie-URL: https://wiki.osdev.org/index.php?title=Bootloader&oldid=30966

````text
{{Disputed}}
A bootloader is a program written to load a more complex [[Kernels|kernel]]. Implementation details are gathered in [[Rolling Your Own Bootloader]]

== What does a boot loader do ==
The boot loader ultimately has to:
* Bring the kernel (and all the kernel needs to bootstrap) into memory
* Provide the kernel with the information it needs to work correctly
* Switch to an environment that the kernel will like
* Transfer control to the kernel

On the x86, the boot loader runs in [[Real Mode]]. Consequently it has easy access to BIOS resources and functions. Therefore it's a good place to perform memory map detection, detection of available video modes, loading of additional files, etc. The boot loader will collect this information and present it in a way the kernel will be able to understand.

=== Loading your kernel ===
The bits of your kernel are somewhere on some disk (presumably the booting disk, but this is not mandatory). Question is: where on the disk? Is it a regular file on a [[FAT|FAT-formatted]] partition? Is it a collection of consecutive sectors in the "reserved area" of the FAT file system (in which case you may need a dedicated tool to format the disk and install the kernel on it)? Or is the disk / partition simply left unformatted and the kernel pasted directly with a disk image tool?

All the above options are possible. Maybe the one I'd choose myself would be to reserve enough space on a FAT file system to store the list of sectors used by the kernel file. The field reserved sectors in the BPB is a perfect place for this. The "advantage" of being fully-FAT is that you don't need to re-write the bootsector every time you rewrite the kernel.

What needs to be loaded mainly depends on what's in your kernel. Linux, for instance, requires an additional 'initrd' file that will contain the 'initialization process' (as user level). If your kernel is modular and if Filesystems are understood by some modules, you need to load the modules along with the kernel. Same goes for 'microkernel services' like disk/files/memory services, etc.

=== Giving the kernel its information ===

Some kernels require some extra information to run. For example, you'll need to tell Linux the root partition to start from. Pretty useful information to have is a map of the address space - effectively a map of where physical memory is and where it's not. Other popular queries regard video modes.

In general, anything that involves a BIOS call is easier to do in [[Real Mode]], so better do them while in real mode than trying to come back to real mode for a trip later.

=== Establishing an environment ===
Most kernels require protected mode. For these kernels you'll have to 
* Enable [[A20 Line|A20]]
* Load a [[GDT]]
* Enter [[Protected mode]]
before giving control to the kernel.

It's common for the loader to keep interrupts disabled (the kernel will enable them later when an IDT is properly set up).

''Note: take time to think about whether or not you'll enable paging here. Keep in mind that debugging paging initialization code without the help of exception handlers may quickly become a nightmare!''

== Bootloader design ==
Virtually any bootloader follows a common design.

=== Single Stage Bootloader ===
A single stage bootloader consists of a single file that is loaded entirely by the BIOS. This image then performs the steps described above to start the kernel.
However, on the x86 you are usually limited to 512 bytes for a first stage (An exception is no-emulation [[El-Torito]]), which is not much. Also, a lot of this size may be dedicated to bios structures and [[FAT]] headers, which leaves even less space to work with

=== Two-Stage Bootloader ===
A two-stage bootloader actually consists of two bootloaders after each other. The first being small with the sole purpose of loading the second one. The second one can then contain all the code needed for loading the kernel. [[GRUB]] uses two (or arguably, three) stages.

=== Mixed Bootloader ===

Another way to avoid the 512-byte barrier is to split the bootloader in two parts, where the first half (512 bytes) can load the rest. This can be achieved by inserting a '512-bytes' break in the ASM code, making sure the rest of the loader is put after the bootsector.

== Booting multiple OSes ==

The easiest way to boot another OS is a mechanism called chainloading. Windows stores something akin to a second-stage bootloader in the boot sector of the partition it was installed in. When installing Linux, writing e.g. LILO or GRUB to the partition boot sector instead of the MBR is also an option. Now, the thing your MBR bootsector can do is to relocate itself (copying from 0x0000:0x7c00 to, traditionally, 0x0060:0x0000), parse the partition table, display some kind of menu and let the user choose which partition to boot from. Then, your (relocated) MBR bootsector would load that partition boot sector to 0x0000:0x7c00, and jump there. The partition boot sector would be none the wiser that there already was a bootsector loaded before, and could actually load yet another bootsector - which is why it's called chainloading.

You see that with displaying a menu in some intelligible way and accepting keystrokes, such a multi-option bootloader can get quite complex rather quickly. We didn't even touch the subject of booting from extended partitions, which would require sequentially reading and parsing multiple extended partition tables before printing the menu.

Taken to the extreme, bootmanagers like that can become as complex as a simple OS, [[GRUB]] being a good example: It offers reading from various filesystems, booting [[Multiboot]] kernels, chainloading, loading [[initrd]] ramdisks etc. etc.

== See Also ==

=== Articles ===
* [[BootProg]] is able to load COM/MZ programs from a FAT12/16/32-formatted media
* [[Bootf]] is a small FAT12 floppy bootloader for protected mode OS images
* [[Gujin]] is a GPLed bootloader for the PC
* [[GRUB]] is a huge, bloated Grand Unified Bootloader, used by many OSes
* [[BOOTBOOT]] for booting 64 bit kernels on BIOS, UEFI, El Torito CDROM etc.
* [[Limine]] is a bootloader capable of natively booting 64-bit kernels and Linux
* [[SysLinux]] is the Linux kernel loader
* [[Rolling Your Own Bootloader]]

=== Threads ===

=== External Links ===
* [http://viralpatel.net/taj/tutorial/hello_world_bootloader.php Writing Hello World Bootloader] ([http://web.archive.org/web/20140729084135/http://viralpatel.net/taj/tutorial/hello_world_bootloader.php Cached version])
* [http://www.cs.cmu.edu/~410-s07/p4/p4-boot.pdf "Writing a Bootloader from Scratch"] from Carnegie Mellon Computer Science Department

[[Category:OS theory]]
[[Category:Booting]]
[[Category:Bootloaders]]
[[de:Bootloader]]
````

<a id="w001683"></a>

## W001683 — Talk:Bootloader

Bron-ID: W001683
Namespace: Talk (1)
Revisie: 30967 | Datum: 2026-09-23T16:03:30Z
Bron: https://wiki.osdev.org/Talk:Bootloader
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3ABootloader&oldid=30967

````text
== Theory or Practice ==
I removed most of the tutorial bits, and appended some theory. This means this version is more educational than instructional. Does someone think the extra code snippets and comments should be put back or be left as an exercise for the reader (A lot of the extra information is about [[FAT]] and [[real mode]], as well as a few others, effectively giving doubled content). Suggestions? - [[User:Combuster|Combuster]] 09:48, 17 December 2006 (CST)

== Renaming this article or updating it ==

I feel like this article should either be updated (to at least include UEFI or maybe other firmwares), or renamed to BIOS bootloader or something like that, since it assumes BIOS (which is obsolete) and gives people a wrong idea about things; namely, I've seen people in discussions refer to this article specifically and still deciding to start with writing a BIOS bootloader, with the intent of targeting "modern" (made since 2012, or earlier) PCs. --[[User:Mishakov|Mishakov]] ([[User talk:Mishakov|talk]]) 22:16, 22 September 2026 (UTC)

I agree, a renaming or a large update (started in someones user namespace perhaps) to include UEFI would be in order. I might start work on revisions that do so sooner or later. --[[User:anotheridiot|Anotheridiot]] ([[User talk:anotheridiot|talk]]) 16:03, 23 September 2026 (UTC)
````

<a id="w001684"></a>

## W001684 — Talk:A20 Line

Bron-ID: W001684
Namespace: Talk (1)
Revisie: 29351 | Datum: 2025-01-25T20:41:18Z
Bron: https://wiki.osdev.org/Talk:A20_Line
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AA20_Line&oldid=29351

````text
== Contents ==
I think this topic needs some practical information about the A20 line. (Added stub tag). - [[User:Combuster|Combuster]] 16:12, 17 December 2006 (CST)

:There is some information about it on OS FAQ Wiki... I will adapt it when I have the time... [[User:JJeronimo|JJeronimo]] 14:41, 23 December 2006 (CST)

::The pre-pentium remark seems to be invalid. It has nothing to do with the processor, only with the chipset used. - [[User:Combuster|Combuster]] 08:33, 24 December 2006 (CST)

:::Yes, yes... on my old 486 DX it worked... [[User:JJeronimo|JJeronimo]] 11:29, 24 December 2006 (CST)

"Don't forget to restore the original bootsector identifier." Why is it important? [[User:Adek336|Adek336]] 21:44, 1 August 2008 (UTC)

Since we are talking about the OS-specific bootloader here, it's not. The OS would know (and probably doesn't care) that the memory image of its own bootloader was slightly self-modified. If the OS happens to boot from a generic bootloader, then the OS had best not assume anything about the bootloader except what is in the bootloader's specs.<br>
However, this article definitely needs an additional section about INT15h AX=2400h and 2401h.<br>
Maybe there should also be a mention that the Compaq method, the HP Vectra method, and the Port 0xEE method also exist -- and pointing to external links for descriptions. [[User:Bewing|Bewing]] 20:53, 2 August 2008 (UTC)

===Interesting Info on A20 Line===
Just some facts that should probably be thrown into the mix here.
* The Intel64 Architecture Manual states, "The functionality of A20M# is used primarily by older operating systems and not used by modern operating systems. On newer Intel 64 processors, A20M# may be absent."
* The PC 2001 System Design Guide states, "If A20M# generation logic is still present in the system, this logic must be terminated such that software writes to I/O port 92, bit 1, do not result in A20M# being asserted to the processor."
* The AMD64 Chipset Manuals allows the USB A20 function to be merged with PS2 A20gate through I/O port 64, bit 30.
* The AMD64 Chipset Manuals guarantee I/O port 92, bit 1 to be available for use (if set to 1 A20M# function is disabled.)
* The AMD64 BKDG states, that after entering SMM mode, "A20M# is disabled. A20M# assertion or deassertion have no effect on SMM mode."
The above facts could probably be used to safely assume that all modern computer systems, 2001 and later, support port 92, bit 1 to control the A20 Line. Furthermore, any modern computer system that does not support port 92, bit 1 will already have the A20 Line disabled. [[User:54616e6e6572|54616e6e6572]] 08:06, 12 August 2010 (UTC)
:Just wanted to write again that some systems don't have Gate A20. BTW, I don't know a SINGLE program which requires Gate A20 off. [[User:Alliumnsk|Alliumnsk]] 06:13, 16 August 2017 (CDT)

== Should the Keyboard Controller enabling method be changed? ==

I just noticed- the method for enabling the a20 gate through the keyboard controller uses functions not mentioned anywhere else, namely <code>kyb_wait_until_done</code> and <code>kyb_send_command</code>. Should those be changed?
````

<a id="w001685"></a>

## W001685 — Talk:A20Line

Bron-ID: W001685
Namespace: Talk (1)
Revisie: 2414 | Datum: 2006-12-17T22:14:13Z
Bron: https://wiki.osdev.org/Talk:A20Line
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AA20Line&oldid=2414
Doorverwijzing: Talk:A20 Line

````text
#REDIRECT [[Talk:A20 Line]]
````

<a id="w001686"></a>

## W001686 — A20Line

Bron-ID: W001686
Namespace: Main (0)
Revisie: 2416 | Datum: 2006-12-17T22:15:05Z
Bron: https://wiki.osdev.org/A20Line
Revisie-URL: https://wiki.osdev.org/index.php?title=A20Line&oldid=2416
Doorverwijzing: A20 Line

````text
#REDIRECT [[A20 Line]]
````

<a id="w001689"></a>

## W001689 — Talk:PCI

Bron-ID: W001689
Namespace: Talk (1)
Revisie: 10025 | Datum: 2010-04-09T07:05:19Z
Bron: https://wiki.osdev.org/Talk:PCI
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3APCI&oldid=10025

````text
== Configuration Methods ==
Does somebody know about the difference between configuration methods? #1 seems standard but there is minimal info about the second or the differences. Additions welcome - [[User:Combuster|Combuster]] 08:47, 18 December 2006 (CST)
: Configuration Mechanism #2 is the enhanced configuration mechanism provided for PCI Express. You can now read about it in the lovely new [[PCI Express]] article. --[[User:54616e6e6572|54616e6e6572]] 07:05, 9 April 2010 (UTC)


Is it just me or is the code segment provided a very horrible example. And it doesn't explain what happens with what the pciConfigReadWord return. And the typing of the code is just.. not right. -[[User:Earlz|Earlz]] 03:35, 29 April 2009 (UTC)

== IRQ routing ==
I've noticed that there isn't any information on IRQ routing. Having researched this a little, I found [http://www.microsoft.com/whdc/archive/pciirq.mspx this] page, which contains the specification for the IRQ routing table unless ACPI gets involved. Unfortunately, I don't know enough about the PCI specification to edit some of the information into the page. Could somebody see if they can have a go at this? --[[User:Computafreak|Computafreak]] 16:49, 17 June 2009 (UTC)

== Disclaimer Can Be Removed ==
I have carefully gone through the PCI Local Bus Specification v3.0 and can happily say that all the information contained in the article is correct, so the disclaimer can be removed. -- [[User:54616e6e6572|54616e6e6572]] 18:20, 27 March 2010 (UTC)

== CardBus structure ==
I've added the CardBus structure, based upon [http://faydoc.tripod.com/structures/08/0878.htm this] page, but it's untested at the moment. I think it'll work, but it still needs checking for errors. If anybody has additional information or corrections, please add them --[[User:Computafreak|Computafreak]] 13:08, 28 March 2010 (UTC)
````

<a id="w001690"></a>

## W001690 — Category talk:VGA

Bron-ID: W001690
Namespace: Category talk (15)
Revisie: 2454 | Datum: 2006-12-19T10:30:30Z
Bron: https://wiki.osdev.org/Category_talk:VGA
Revisie-URL: https://wiki.osdev.org/index.php?title=Category_talk%3AVGA&oldid=2454

````text
Should the VGA category be merged with Video? (or subclassed to it?) - [[User:Combuster|Combuster]] 08:50, 18 December 2006 (CST)
:'''Yes.''' Subclassed if there are more articles than the current one, merged otherwise. -[[User:Jhawthorn|Jhawthorn]] 19:31, 18 December 2006 (CST)
Subclassed it for now - I'm currently gathering all the info i need for a [[VGA Hardware]] article. If you think 3 VGA topics is enough to justify a merge, go ahead (the three would probably become [[VGA Resources]] , [[VGA Hardware]] , [[VGA BIOS]]) - [[User:Combuster|Combuster]] 04:30, 19 December 2006 (CST)
````

<a id="w001692"></a>

## W001692 — User talk:Walling/Archive/Discussion1

Bron-ID: W001692
Namespace: User talk (3)
Revisie: 8160 | Datum: 2009-06-04T14:34:58Z
Bron: https://wiki.osdev.org/User_talk:Walling/Archive/Discussion1
Revisie-URL: https://wiki.osdev.org/index.php?title=User_talk%3AWalling%2FArchive%2FDiscussion1&oldid=8160

````text
==Main Page title==
I have gotten your code to work on [http://www.johnhawthorn.com/ my mediawiki installation]. It seems to work well enough.

On this wiki I assume that user javascript is disallowed until chase adds
 $wgAllowUserJs = true;
to <tt>LocalSettings.php</tt> (that did the trick on my install).

Or alternatively, adding
 function hideMainPageTitle(e){
    var mainPageTitle = "Main Page";
    var headings = document.getElementsByTagName("h1");
    for (var i = 0; i < headings.length; i++){
       var heading = headings[i];
       if (heading.className == "firstHeading"){
          if ((heading.innerHTML == mainPageTitle) && (document.getElementById("contentSub").innerHTML  == "")){
             heading.style.display = "none";
             document.getElementById("siteSub").style.display = "none";
             document.getElementById("contentSub").style.display = "none";
          }
          break;
       }
    }
 }
 
 addOnloadHook(hideMainPageTitle);

to <tt>MediaWiki:monobook.js</tt> will enable it for all users. (currently used on my installation)

I'll leave you to decide what to do currently. I'm personally a little uncomfortable with looping through every h1 tag. Seems to be begging for an exploit.
-[[User:Jhawthorn|Jhawthorn]] 18:34, 18 December 2006 (CST)

: Thanks, for the testing it. Then I know it could work. Maybe you could test [http://www.osdev.org/phpBB2/viewtopic.php?p=86298#86298 these as well]. There is a php implementation (which I think is the best solution) and another in javascript, which is used by enwiki. The javascript solution works using a combination of javascript, which locates the main page, and then writes some css code. Maybe it is better than the other javascript solution, but still I think it should be done serverside. The php solution is a patch file, but I don't know how to apply it. It could be cumbersome to edit the files yourself. --[[User:Walling|Walling]] 02:11, 19 December 2006 (CST)
::The php patch works perfectly. It does seem to be the better solution. -[[User:Jhawthorn|Jhawthorn]] 00:45, 25 December 2006 (CST)
````

<a id="w001693"></a>

## W001693 — User:Jhawthorn/linkconvert

Bron-ID: W001693
Namespace: User (2)
Revisie: 2442 | Datum: 2006-12-19T03:22:59Z
Bron: https://wiki.osdev.org/User:Jhawthorn/linkconvert
Revisie-URL: https://wiki.osdev.org/index.php?title=User%3AJhawthorn%2Flinkconvert&oldid=2442

````text
A little python script I threw together to convert links to the MegaTokyo board to the new phpbb board on osdev.org
Accepts anything with a <tt>threadid=</tt> or <tt>start=msg</tt> followed by a number including a full URL.

==Standalone version==
:''N.B. This still requires python, just not <tt>topic_map.txt</tt> or <tt>message_map.txt</tt>''
Download from http://www.osdev.org/mediawiki/images/c/ce/Linkconvert.zip

==Using external files==
This requires the files <tt>topic_map.txt</tt> and <tt>message_map.txt</tt> to be in the current directory. They are available from zip archives here: http://www.osdev.org/phpBB2/viewtopic.php?p=83433#83433

<pre>
#!/usr/bin/env python

"""
A little python script I threw together to convert links to the MegaTokyo board to the new phpbb board on osdev.org
Accepts anything with a "threadid=" or "start=msg" followed by a number including a full URL.
"""

import re
import sys

topics = file("topic_map.txt").read()
messages = file("message_map.txt").read()

def search(f, number):
	regex = re.compile("".join((number, ",\d+")),)
	s = re.search(regex, f)
	if s:
		return s.group()[len(number)+1:]
	else:
		return None

data = sys.stdin.readline()
while data:
	s1 = re.search(re.compile("start=msg\d+"), data)
	s2 = re.search(re.compile("threadid=\d+"), data)
	if s1:
		s = search(messages, s1.group()[9:])
		print "".join(("http://www.osdev.org/phpBB2/viewtopic.php?p=", s, "#" , s))
	elif s2:
		s = search(topics, s2.group()[9:])
		print "".join(("http://www.osdev.org/phpBB2/viewtopic.php?t=", s))
	data = sys.stdin.readline()

</pre>
````

<a id="w001694"></a>

## W001694 — File:Linkconvert.zip

Bron-ID: W001694
Namespace: File (6)
Revisie: 2441 | Datum: 2006-12-19T03:18:31Z
Bron: https://wiki.osdev.org/File:Linkconvert.zip
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3ALinkconvert.zip&oldid=2441

````text
Standalone version of [[User:Jhawthorn|Jhawthorn]]'s python [[User:Jhawthorn/linkconvert|linkconvert]] script.
````

<a id="w001695"></a>

## W001695 — Talk:AMD K6 WriteBack Optimisations

Bron-ID: W001695
Namespace: Talk (1)
Revisie: 2449 | Datum: 2006-12-19T08:24:56Z
Bron: https://wiki.osdev.org/Talk:AMD_K6_WriteBack_Optimisations
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AAMD_K6_WriteBack_Optimisations&oldid=2449

````text
== Introduction ==

What is AMD K6 WriteBack Optimisations? I think it could be nice with a short introduction. --[[User:Walling|Walling]] 02:24, 19 December 2006 (CST)
````

<a id="w001696"></a>

## W001696 — File:VGA overview.gif

Bron-ID: W001696
Namespace: File (6)
Revisie: 2455 | Datum: 2006-12-19T11:16:22Z
Bron: https://wiki.osdev.org/File:VGA_overview.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AVGA_overview.gif&oldid=2455

````text
General layout of VGA mechanics
````

<a id="w001697"></a>

## W001697 — VGA Hardware

Bron-ID: W001697
Namespace: Main (0)
Revisie: 30577 | Datum: 2026-05-17T09:47:51Z
Bron: https://wiki.osdev.org/VGA_Hardware
Revisie-URL: https://wiki.osdev.org/index.php?title=VGA_Hardware&oldid=30577

````text
Even though VGA is old, many modern graphics cards are compatible with it, including NVidia and AMD (ATI) cards. This can make writing a VGA driver rather attractive. The amount of compatibility varies however, so never assume a card is compatible without proper hardware detection. 
Apart from real machines, several emulators and virtual machines provide VGA emulation:  [[Bochs]], [[QEMU]] and [[Microsoft Virtual PC]] to name a few. After 2011 video card manufacturers begun to drop VGA compatibility in favour of [[GOP]] under [[UEFI]] (VirtualBox UEFI and TianoCore both supports that).
{{Video warning}}


== Overview ==

While the VGA chip is quite a simple piece of hardware compared to modern video equipment, it is possibly one of the more complicated devices to program for, and especially in the old days knowing your way around this particular device was sufficient for establishing quite a reputation. While currently a legacy device, it is a good place to begin practicing your video driver skills. While a full-blown VGA driver might make an USB controller look trivial, there are fortunately many shortcuts available for taking.

=== What's not covered ===
While this page tries to be a complete overview on what the VGA can do, it does not fully cover the whole set of graphics. After all, a video card only turns bytes in its memory into a signal on the connector on its backside. Determining what bytes to put in memory is only barely touched in the wiki in general — there are examples of plotting pixels and setting individual characters but your OS will determine what pixels are formed by an image and which characters are part of your title screen. On the remote end, monitors have their own way of dealing with signals. A lot of those settings dictated by monitors are needed by the video card, and each resolution comes with its own set of settings. You can find out your own set of settings by [[Video Signals And Timing|using a set of equations]], but you can skip that step and reuse one of the examples provided at the [[VGA_Hardware#Sample_Register_Settings|example settings]] instead. The [[VGA_Hardware#The_CRT_Controller|CRTC chapter]] explains them in detail.

=== Getting started ===
There's a fair share of [[VGA Resources|modesetting code available]] around the web. The basic steps involve calculating the needed register values, writing them to the VGA, then continue with drawing. You'll need: 
* Port I/O: The VGA needs 8-bit read/writes, and 16-bit writes.
* MMIO: The VGA uses uncached byte accesses to 0xA0000-0xBFFFF. In several cases, larger writes are also allowed.
* Functions to [[VGA_Hardware#VGA_Registers|read and write registers]] for each VGA component - since there are many more registers than there are ports you will need a wrapper for this.
* A structure that contains the VGA display settings. For a nice list of things you want to set, you can use the example register settings part. Keep in mind that you might also want a structure for things that change during drawing, such as colours and offsets.
* A function that writes that structure to the device
* A function that fills out that structure. You can also use a hardcoded structure initially.

=== Hardware components ===

The VGA can be divided in several parts. Historically, the predecessor of the VGA - the EGA - had several chips to perform each part in the system. These chips could be configured to your liking using the I/O Bus. On the VGA, these have been merged into one chip (with the exception of the DAC).

The following diagram shows which units are responsible for which parts:

[[Image:VGA overview.gif|Overview of VGA Hardware|325px]]

This diagram is, however, a simplification for the ease of programming, and should not be considered correct.

== VGA Registers ==
The VGA has a lot (over 300!) internal registers, while occupying only a short range in the I/O address space. To cope, many registers are indexed. This means that you fill one field with the number of the register to write, and then read or write another field to get/set the actual register's value. 

All registers are accessed as 8 bit. The parts of a register that are not used should officially be preserved, although a lot of programs simply set them to zero. However, not all fields present in the VGA are documented here, so you will either look up a different reference, or preserve the undocumented fields. 

In the documentation below, a port number and possibly an index is provided. The port is usually the base port for indexed registers, or the actual port for single registers. 

Note that [[PCI]] boards do *not* report the VGA addresses in their configuration space, and that the addresses can not be remapped. It is therefore not possible to properly operate two cards in VGA mode at the same time.

=== Port 0x3C0 ===
This port is a confusing one: you write both the index and data bytes to the same port. The VGA keeps track of whether the next write is supposed to be the index or the data byte. However, the initial state is unknown. By reading from port 0x3DA it'll go to the index state. To read the contents, feed the index into port 0x3C0, then read the value from 0x3C1 (then read 0x3DA as it is not defined whether the VGA expects a data byte or index byte next).

=== Port 0x3C2 ===
This is the miscellaneous output register. It uses port 0x3C2 for writing, and 0x3CC for reading. Bit 0 of this register controls the location of several other registers: if cleared, port 0x3D4 is mapped to 0x3B4, and port 0x3DA is mapped to 0x3BA. For readability, only the first port is listed and bit 0 is assumed to be set.

=== Port 0x3C4, 0x3CE, 0x3D4 ===
These are the most used indexed registers. The index byte is written to the port given, then the data byte can be read/written from port+1. Some programs use a single 16-bit access instead of two byte accesses for writing, which does effectively the same. (take care of byte ordering when doing so)

Port 0x3D4 has some extra requirements - it requires bit 0 of the '''Miscellaneous Output Register''' to be set before it responds to this address (if cleared, these ports appears at 0x3B4). Also, registers 0-7 of 0x3D4 are write protected by the protect bit (bit 7 of index 0x11)

=== Port 0x3C6 ===
Port 0x3C6 only contains the DAC Mask Register, which can easily be accessed by a simple read/write operation on this port. Under normal conditions it should contain 0xff.

=== Port 0x3C8 ===
Port 0x3C8, 0x3C9 and 0x3C7 control the DAC. Each register in the DAC consists of 18 bits, 6 bits for each color component. To write a color, write the color index to port 0x3C8, then write 3 bytes to 0x3C9 in the order red, green, blue. If you want to write multiple consecutive DAC entries, you only need to write the first entry's index to 0x3C8 then write all values to 0x3C9 in the order red, green, blue, red, green, blue, and so on. The accessed DAC entry will automatically increment after every three bytes written. To read the DAC entries, write the index to be read to 0x3C7, then read the bytes from port 0x3C9 in a similar fashion (as with writing, the index will increment after every three bytes read)

== Video Memory Layout ==
The video memory consists of four 'planes' (individual units) of memory, each with a size of 64KB, giving the VGA 256k of video memory. Connected to it is the Sequencer, which interprets this memory to generate colors which are fed to the subsequent stages. The way colors are organized in this memory mainly depends on the color depth. 

Specific details about how memory is accessed from the host is can be found by reading about the Graphics Controller, detailed information about video memory is rendered can be found by reading about the Sequencer.

=== Memory Layout in 16-color graphics modes ===
16 colors means there are 4 bits used per color. The VGA has four planes, and for each pixel, and each plane holds one bit of each pixel drawn. Since the information for each pixel is scattered over four memory locations, this is the most difficult memory model.

Bit 7 of each address contains information about the first pixel, Bit 6 has information about the next pixel, and so on.

Plane 0 contains the first bits of all pixels, Plane 1 the second bits and so on.

Example:

{| {{Wikitable}}
|-
!
! colspan="8" | one byte
|-
! Bits
! 7
! 6
! 5
! 4
! 3
! 2
! 1
! 0
|-
! Plane 0
| 0
| 0
| 0
| 0
| 1
| 1
| 1
| 1
|-
! Plane 1
| 0
| 0
| 1
| 1
| 1
| 1
| 0
| 0
|-
! Plane 2
| 0
| 1
| 1
| 0
| 0
| 1
| 1
| 0
|-
! Plane 3
| 0
| 1
| 0
| 1
| 0
| 1
| 0
| 1
|- 
! Colors displayed
| 0000 (0)
| 1100 (12)
| 0110 (6)
| 1010 (10)
| 0011 (3)
| 1111 (15)
| 0101 (5)
| 1001 (9)
|}

The Plane Write Enable register is used to choose the plane to be written, then the memory can be written by accessing the corresponding address in memory.

=== Memory Layout in 256-color graphics modes ===
In this mode, each byte of video memory describes exactly one pixel. Pixels are generated by increasing address in linear mode, with all colors taken from plane 0. In planar mode (Also known as Mode X) each address describes 4 consecutive pixels, one from each plane. Plane 0 describing the first pixel, plane 1 the next, and so on. Technically speaking this is what always happens, but the standard 320x200x256 mode "chains" the planes such that 2 lowest order bits select the plane and the memory thus appears linear.

In linear mode, each byte in host memory corresponds to one pixel on the display, making this mode very easy to use. Mode X requires the use of Plane Write Enable register to select the plane to be written.

=== Memory Layout in text modes ===
In text mode, the screen is divided into character cells rather than pixels.
Only three of the four planes are actually in use. Plane 0 contains the character codes for each cell, Plane 1 contains the respective attributes. 

Plane 2 contains the font data. For each of the 256 available characters this plane has 32 bytes reserved. Each byte represents one horizontal cross section through each character. The first byte of each group defines the top line, each next byte describes the rows below it. For every set bit, the foreground color is used, For every cleared bit, the background color is used.

Although 32 bytes are reserved for each character, only 16, 14, or 8 of them are commonly used, depending on the character height.

Planes 0 and 1 are accessible from the host by writing to the video memory range. Plane 0 is accessed on even addresses, plane 1 is accessed on odd addresses, with each consecutive 16-bit value describing the next character. Accessing plane 2 to change fonts requires [[VGA Fonts|changes in addressing logic]].

=== Memory Layout in 4-color modes ===
The CGA was limited to 4 concurrent colors, with two bits each. The EGA adds two extra bits by adding a pair of extra planes, increasing from the old two to the current four planes per pixel. If you want a 4-color mode that means you just should not touch planes 2 + 3. 

'''Todo: determine the b/w/d, shift mode and odd/even mode for CGA compatibility (guesstimated at word mode, interleaved shift, odd/even enabled, i.e. equivalent to text mode except for the alphanumeric bit)'''

== The Graphics Controller ==
The Graphics Controller (Abbreviated to GC) is responsible for directing memory reads and writes to and from video memory.

The memory consists of four planes of 64k on a standard VGA. Each read and write operation selects one address within all of these planes, then operates on all four planes. This means that for each byte of data written, four bytes of video memory might potentially be changed.

Apart from a few standard modes, the implementation of various control bits vary between implementations. However, the exact details can be probed by performing writes in the mode to check, then reading it out in planar mode.

These modes seem to be consistent among all VGA compatibles and emulators I've tested:
* Mode 3 (Text mode) (right now I can only set it by keeping a state dump of VGA registers of the old mode)
* Mode 11h (Planar 16 color mode)
* Mode 13h (Linear 256 color mode) 
* Mode X (Planar 256 color mode)

I have yet to write code that enters the following standard modes on all hardware

* Mode 04h (4 color mode) - Not tried, should be achieved by setting interleaved shift mode and ignoring planes 2 & 3.

The Graphics Controller consists of of some addressing logic, and some specific read/write logic.

The Addressing logic controls the address in video memory to be accessed, and to some extent, the individual pages within it.

The Read/Write logic controls which planes of the memory are actually read or written, and how these values relate to the value being  sent by the CPU.

[[Image:VGA gc addressdecode.gif|Datapaths in the Graphics Controller]]

=== Addressing Logic ===

Registers involved:
{| {{Wikitable}}
|-
! Register Name
! Port
! Index
! 7
! 6
! 5
! 4
! 3
! 2
! 1
! 0
|-
| Sequencer Memory Mode Register
| 0x3C4
| 0x04
|
|
|
|
| Chain 4
| Odd/Even Disable
| 
|
|}

==== Chain 4 Bit ====

The Chain-4 bit changes accesses to video mode from a planar mode when clear to a linear mode when set. While under common circumstances this bit is emulated properly, the way this bit actually works is however very different among implementations (especially emulators) and can have strange effects if you are unaware of it. What works in all cases, is if chain-4 matches the other settings that are common for established modes (i.e. if you enable chain-4, also make sure the other registers match the expected values for mode 13). Here is a list of differences between various implementations of Chain-4. Real hardware has so far proven to be consistent with the officially documented behavior.

{| {{Wikitable}} 
|-
|
! Bochs
! Qemu
! Virtual PC
! ATI Card (X300)
! NVidia (GeForce 6 6150)
|-
! Chain 4 has effect on writes
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
| {{Yes}}
|-
! Chain 4 has effect on output
| {{Yes}}
| {{No}}
| {{No}}
| {{No}}
| {{No}}
|-
! Plane Write Enable takes effect on writes
| {{No}}
| {{Yes}}
| {{No}}
| {{Yes}}
| {{Yes}}
|}

In bochs + VPC, as well as the hardware tested, Chain4 writes occur to this address:

 plane  = addr & 0x0003; // lower bits
 offset = addr & 0xfffb; // rest of the bits. multiples of 4 wont get written

In qemu writes go here:

 plane  = addr & 0x0003; // lower bits
 offset = addr >> 2;     // only the first 16k of each page gets written.

Note the fact that in Bochs, Chain-4 alters the physical display of the screen (equivalent to what doubleword mode normally does). This makes Bochs possibly troublesome since you only need to toggle this bit to enter Mode-X, while real hardware also requires that you change doubleword mode into byte mode.

==== Odd/Even Disable Bit ====
Seems to be a Don't Care bit in bochs+qemu

VPC generates some form of echo:

 plane1 = addr & 0x0001; // lowest bit
 plane2 = plane1 + 2;    // pick the other odd/even plane
 offset = addr & 0xfffe; // generate the offset
 write (plane1, offset); // write to the plane
 write (plane2, offset); // write to the other plane

This matches the NVidia card, however my ATI card (and somewhat older, my V2x00 board) behaves slightly different (its pretty close though):

 offset = addr & 0xfff'''f'''; // generate the offset

''' There are probably some more things involved here, todo '''

=== Read/Write logic ===

The Read/Write Logic performs several operations on the written/read data, and a set of internal registers called the latches. Reading from video memory loads these latches with the value emitted by video memory. Write operations use the latches as an additional data source, apart from the data written from the host processor.

The read/write logic has several different operation modes. These can be chosen by setting the Graphics Mode register. The VGA has four write modes and 2 read modes, which can be set independently. By default, the VGA operates in read mode 0 and write mode 0 in such a fashion that all written data goes straight to memory, and read data from each plane is ORed together.

Registers involved:
{| {{Wikitable}}
|-
! Register Name
! Port
! Index
! 7
! 6
! 5
! 4
! 3
! 2
! 1
! 0
|-
| Graphics Mode Register
| 0x3CE
| 0x05
|
|
|
|
| Read Mode
| 
| colspan="2" | Write Mode
|-
| Map Mask Register
| 0x3C4
| 0x02
|
|
|
|
| colspan="4" | Memory Plane Write Enable
|-
| Enable Set/Reset Register
| 0x3CE
| 0x01
|
|
|
|
| colspan="4" | Enable Set/Reset
|-
| Set/Reset Register
| 0x3CE
| 0x00
|
|
|
|
| colspan="4" | Set/Reset Value
|-
| Data Rotate Register
| 0x3CE
| 0x03
|
|
|
| colspan="2" | Logical Operation
| colspan="3" | Rotate Count
|-
| Bit Mask Register
| 0x3CE
| 0x08
| colspan="8" | Bit Mask
|}

'''Todo: Study effects of multibyte reads/writes on logic and latch operation'''

==== The Latches ====
Probably the most interesting part of the VGA's internal wiring is the presence of the data latches. In the old times, the VGA could only accept 8 bits at a time. The latches hold 4x8 bits, and are used as a temporary register for VGA reads and writes. By putting that register to good use, a DOS-era programmer could well exceed the data transfer rate that the 8-bit bus was capable of, and instead use the 32-bit pipeline onboard the video card.

The latches are written to whenever a load from video memory occurs. An address is supplied to video memory, which emits the 4 bytes, one for each plane, into the latches. From there, the video card determines what to send to the CPU. The latches are used again when writing to video memory. Together with the various read and write modes, the latches allowed video-to-video transfers, pattern and raster operations, as well as supplying original data for doing partial writes.

==== Write Mode 0 ====
Write mode 0 is the standard write mode. 

[[Image:VGA gc write0.gif|Write Mode 0 Logic]]

When a byte is written, it follows the following steps
* The input byte is rotated right by the amount specified in Rotate Count, with all bits shifted off being fed into bit 7
* The resulting byte is distributed over 4 separate paths, one for each plane of memory
* If a bit in the Enable Set/Reset register is clear, the corresponding byte is left unmodified. Otherwise the byte is replaced by all 0s if the corresponding bit in Set/Reset Value is clear, or all 1s if the bit is one.
* The resulting value and the latch value are passed to the ALU
* Depending of the value of Logical Operation, the following operation is performed: 
{| {{Wikitable}} 
|- 
! Value of Logical Operation 
! Result 
|- 
| 0 (0x00)
| The byte from the set/reset operation is forwarded
|-
| 1 (0x08)
| Both inputs are ANDed together
|-
| 2 (0x10)
| Both inputs are ORed together
|-
| 3 (0x18)
| Both inputs are XORed together
|}
* The Bit Mask Register is checked, for each set bit the corresponding bit from the ALU is forwarded. If the bit is clear the bit is taken directly from the Latch.
* The Memory Plane Write Enable field is ANDed with the input from the address logic. For each set bit in the result, the corresponding plane is loaded with the result.

==== Write mode 3 ====

Write mode 3 can, among others, be used for transparent writes with a constant color

[[Image:VGA gc write3.gif|Write mode 3 logic]]

'''Todo: thorough testing. This is currently an unverified interpretation of existing docs'''

When a byte is written:
* The input byte is rotated right by the amount specified in Rotate Count, with all bits shifted off being fed into bit 7
* The resulting value is ANDed with the Bit Mask Register, resulting in the bit mask to be applied.
* Each plane takes one bit from the Set/Reset Value register, and turns it into either 0x00 (if set) or 0xff (if clear)
* The computed bit mask is checked, for each set bit the corresponding bit from the set/reset logic is forwarded. If the bit is clear the bit is taken directly from the Latch. The result is sent towards memory.
* Finally, The Memory Plane Write Enable field and the input line from the address logic are ANDed together. The bits that remain set are the planes that are actually written.

''' Todo: more write modes, read modes '''

== The Sequencer ==
The Sequencer is responsible to convert video memory to color indexes. Like the graphics controller, it has some special addressing logic, which is designed to iterate over the memory in a sensible manner to produce images out of video memory data.

The Sequencer either operates in text (alphanumeric) mode or graphics mode

=== Alphanumeric Mode ===
In alphanumeric mode the four planes are assigned distinct tasks. Plane 0 contains character data, while plane 1 contains Attribute data. In a standard text mode, these planes are interleaved into host memory. Plane 2 holds the font data. When displaying text data, the sequencer loads the character/attribute pair for the current set of eight pixels, after which it uses the value for byte 0 to look up the corresponding character in plane 2 and adds the character line to get the font data needed. It then pops out the bits from MSB to LSB, generating the chosen foreground color when a 1 is encountered, and the background color when a 0 is encountered.

In typical text modes, the stored font is not directly accessible and needs some [[VGA Fonts|changes in addressing logic]] to be read or written.

'''TODO: schematics'''

'''TODO: testing'''

=== Graphics Mode ===
In graphics mode, operations can be divided into two steps: address computation and shift logic. The sequencer computes an address, then reads out the four planes from that address, and generates 8 pixels from these values.
The shift logic has three operating modes: single, interleaved and 256-color shift. Although the VGA 'supports' various color depths, these are basically varations on 16-color modes.

==== Shift modes ====
While going through memory, the sequencer reads in a 4 bytes at a time from each of the four planes, then outputs 8 pixel colors. The VGA has three distinct modes of grouping this data into pixel values. The setting depends on two bits in a VGA register: the 256-color shift and interleaved shift bits. when both are off, Single shift mode is selected, otherwise the corresponding mode is used (256 color shift mode takes precedence over interleaved shift mode)

{| {{Wikitable}}
|-
! Register Name
! Port
! Index
! 7
! 6
! 5
! 4
! 3
! 2
! 1
! 0
|-
| Graphics Mode Register
| 0x3CE
| 0x05
|
| 256-Color Shift
| Interleaved Shift
| 
| 
| 
|
|
|}

* Single shift mode
:This mode is used in 16 color modes. For each pixel, one bit is popped off each plane and put together to form the value of a pixel. An example is given in [[VGA Hardware#Memory Layout in 16-color graphics modes|Memory Layout in 16-color graphics modes]]

* Interleaved Shift Mode
:This mode makes 4-color modes relatively easy: 2 bits are popped off the most significant side of plane 0. The same is done with plane 2, which become the most significant bits (in 4-color modes, these are zero) After 4 pixels being popped from planes 0 and 2, the same is done with plane 1 and 3, until all 8 pixels have been generated. 

* 256-Color Shift Mode 
:This mode causes 4 bits to be popped of each time. Plane 0 gives the first two pixels, Plane 1 the next two and so on. However, it is not defined in which order this happens. Because this mode is normally used solely in 256-color modes where the color logic will merge two 4-bits pixels together to form one 8-bit pixel, the communication inbetween is not certain. However, the bits can only be shifted out one of two possible sides, and supporting two possibilities can be overseen. Another problem to this method is, you can not detect which method is used without user intervention or keeping a list. Either way, this method either shifts left (from the msb) or right (from the lsb). If you know the ordering of your video card, you can create a linear 16-bit color mode.

=== Address Calculation ===

For each group of pixels, the sequencer calculates the address in video memory where to load the data from. This address is calculated in three steps:

First the starting address of video memory is calculated: '''TODO'''

After that a scanline is rendered. The first address is read and the contents is split into pixels. Then the address is incremented by either 1, 2 or 4 for the next set of pixels until the scanline completes (the consequence of this is that each scanline is a multiple of eight pixels wide). The increment depends on wether the VGA is operating in "byte mode", "word mode", or "doubleword mode". These can be set using two bits: (doubleword modes takes precedence over byte/word mode)
'''TODO: Registers'''

Once a scanline is complete, the original value at the start of the scanline is loaded, and the scanline counter is incremented. The VGA then does one of the following:
* Add the virtual width to the offset (and going to the next sequence of pixel data in memory). You can change the virtual width to change the amount of free data between scanlines, which can be useful for scrolling screens.
* Leave the value unchanged (and draw the same scanline again, identically). This is done when doublescanning '''TODO'''
* Reset the address to 0 (and start rendering from a different location in video memory) This can be used to create splitscreens '''TODO'''

== Color Logic ==
This block revolves around the Attribute Controller, Palette RAM and DAC, which are together responsible for generating a color signal out of an index generated by the Sequencer.

'''TODO'''

== The CRT Controller ==
The Cathode Ray Tube Controller, or CRTC, is the unit to create a video signal from the data produced by the DAC. By programming this unit you can control the resolution of your monitor, as well as some hardware overlay and panning effects.

The following diagram gives a quick overview of how the CRTC is generally configured (with register names)

=== Resolution and Timing ===

[[Image:VGA crtc.gif|CRTC output diagram]]

Horizontal timings are based on character clocks (multiples of 8 or 9 pixels), Vertical timings are per-scanline. Since these easily exceed the 255 limit of one byte, the Overflow Register is used to store the high-order bits.

Horizontally and vertically, the area can be separated into four parts: 
* Active Display: The data rendered from memory, from the human perspective this area equals the resolution.
* Overscan: the area around the Active Display. Although commonly black, this can be made visible by changing its colour. By default this is 8 pixels in size on each end.
* Blanking Area: When the CRTC disables the colour output. This is the black area around the screen. By changing this you can move, center, and scale the screen.
* Retrace Period: This area is not generally visible. This is just a signal sent to the monitor to go to the next scanline or the next frame. However, monitors may steal some of this time for the blanking area following it or vice versa. Consequently, the blanking sizes on either side are not equal to accommodate.

==== Registers ====
Registers involved in horizontal timing:
{| {{Wikitable}}
|-
! Register Name
! Port
! Index
! 7
! 6
! 5
! 4
! 3
! 2
! 1
! 0
|-
| Horizontal Total Register
| 0x3D4
| 0x00
| colspan="8" | Horizontal Total
|-
| End Horizontal Display Register
| 0x3D4
| 0x01
| colspan="8" | Horizontal Display End
|-
| Start Horizontal Blanking Register
| 0x3D4
| 0x02
| colspan="8" | Horizontal Blanking Start
|-
| End Horizontal Blanking Register
| 0x3D4
| 0x03
|
| colspan="2" | Horizontal Display Skew
| colspan="5" | Horizontal Blanking End (bits 0..4)
|-
| Start Horizontal Retrace Register
| 0x3D4
| 0x04
| colspan="8" | Horizontal Retrace Start
|-
| End Horizontal Retrace Register
| 0x3D4
| 0x05
| H. Blanking End (bit 5)
| colspan="2" |
| colspan="5" | Horizontal Retrace End
|}

Registers involved in vertical timing:
{| {{Wikitable}}
|-
! Register Name
! Port
! Index
! 7
! 6
! 5
! 4
! 3
! 2
! 1
! 0
|-
| Vertical Total Register
| 0x3D4
| 0x06
| colspan="8" | Vertical Total (bits 0..7)
|-
| Overflow Register
| 0x3D4
| 0x07
| V.Retr.St. (9)
| V.Disp.End (9)
| V.Total (9)
|
| V.Blank.St. (8)
| V.Retr.St. (8)
| V.Disp.End (8)
| V.Total (8)
|-
| Maximum Scan Line Register
| 0x3D4
| 0x09
| colspan="2" | 
| V.Blank.St. (9)
| colspan="5" |
|-
| Vertical Retrace Start Register
| 0x3D4
| 0x10
| colspan="8" | Vertical Retrace Start (bits 0..7)
|-
| Vertical Retrace End Register
| 0x3D4
| 0x11
| colspan="4" |
| colspan="4" | Vertical Retrace End 
|-
| Vertical Display End Register
| 0x3D4
| 0x12
| colspan="8" | Vertical Display End (bits 0..7)
|-
| Vertical Blanking Start Register
| 0x3D4
| 0x15
| colspan="8" | Vertical Blanking Start (bits 0..7)
|-
| Vertical Blanking End Register
| 0x3D4
| 0x16
| colspan="8" | Vertical Blanking End (bits 0..6)
|}

Other registers dealing with timing:
{| {{Wikitable}}
|-
! Register Name
! Port
! Index
! 7
! 6
! 5
! 4
! 3
! 2
! 1
! 0
|-
| Miscellaneous Output Register
| 0x3C2
| -
| colspan="4" | 
| colspan="2" | Clock Select
| colspan="2" |
|-
| Clocking Mode Register
| 0x3C4
| 0x01
| colspan="7" | 
| colspan="2" | 9/8 Dot Mode
|}

==== Timing Model ====

The horizontal timing registers are based on a unit called 'character' (As they match one character in text mode). Each character equals 8 ('''9/8 Dot Mode''' is set) or 9 ('''9/8 Dot Mode''' is clear) pixels. Each scanline contains '''Horizontal Total''' + 5 characters, zero based. '''Horizontal Display End''' tells us the last character that is calculated from memory (i.e. the horizontal resolution in characters minus one). '''Horizontal Blanking Start''' and '''Horizontal Retrace Start''' give us the last character before either period is started. '''Horizontal Blanking End''' and '''Horizontal Retrace End''' need more explanation, as they only contain part of a number. When blanking or horizontal retrace is enabled the significant bits are checked against the character counter, and if these bits match the respective period will be ended. The quick solution is to calculate the appropriate values, compute the last character clock at which each period should be active, then AND it with 0x3F (Blank) or 0x1F (Retrace) to get the register's value. Note that the periods must be between 1 and 63(Blank)/31(Retrace) character clocks. To be safe, there must be at least one character of overscan on each side of the screen to avoid additional artefacts.

The vertical timing is similar, apart from the fact that these registers operate on scan lines (pixels) instead of characters. The '''Vertical Retrace End''' and '''Vertical Blank End''' registers work also similar, although they are different sizes. The Retrace End is 4 bits wide (AND with 0xF, period is 1-15 scanlines), The Blank End size is at least 7 bits (some say its 8, some say its 7),  so the value is computed by ANDing with 0xFF, with the period ranging from 1-127 scanlines. As with horizontal timing, at least one scan line of overscan must be present to avoid possible artefacts.

The clock can be selected using '''Clock Select'''. Only two of four possible clocks are present on all VGAs. A clock of 25MHz is selected when this field is zero, 28MHz is selected when this field equals 1. Some boards have other clocks under values 2 and 3, but you should not write these values unless you know the clock that is there. Note that selecting the 28MHz clock and 9 pixels per pixel results in the same timings as selecting the 25MHz clock and 8 pixels per character, only with different resolutions.

The refresh rates can be calculated as follows:
* Horizontal Refresh Rate = Clock Frequency (in Hz) / total pixels horizontally
* Vertical Refresh Rate = Horizontal Refresh Rate / total scan lines
On a VGA monitor, the horizontal refresh rate should equal 31.25 kHz. Vertically, only 400 and 480 pixel resolutions are used.

If your monitor supports it, you can set virtually any crazy resolution you want. (provided the horizontal resolution is a multiple of 8 or 9 - 8 seems to be common) Most modern monitors allow anything between 400x300 and 800x600 being set this way.

==== Sample timing scheme ====

640x480 (16 colours) uses the following sizes: 
* Timing: 25MHz dot clock, 8 pixels per character
* Totals: 800 pixels horizontally, (100 characters), 524 scan lines
* Active Display: 640x480 (80 characters, 480 scan lines)
* Overscan: 8 pixels (8 scan lines vertically / 1 character clock horizontally) on each side
* Horizontal Retrace: 12 characters (96 pixels)
* Vertical Retrace: 2 scan lines
* Blanking (Left): 2 characters (16 pixels)
* Blanking (Right): 4 characters (32 pixels)
* Blanking (Top): 24 scan lines
* Blanking (Bottom): 2 scan lines
Which should be VGA compatible

== Sample Register Settings ==

These are the register values that can be loaded into the VGA to set a standard mode. Note that you should unlock the CRTC and disable output before loading these registers, and afterwards restoring these to be nice for old monitors that are around.

The pseudocode for changing modes is roughly as follows:

 DisableDisplay   // disable output
 UnlockCRTC       // unlock registers
 
 LoadRegisters    // load registers
 ClearScreen      // clear the screen contents
 LoadFonts        // and for text mode, load fonts 
                  // note that this may need to alter GC settings
                  // so be sure to restore those after that
 
 LockCRTC         // optional: lock the registers again
 EnableDisplay    // make sure there is output

=== List of register settings ===

{| {{wikitable}}
|-
! Register name
! port
! index
! mode 3h (80x25 text mode)
! mode 12h (640x480 planar 16 color mode)
! mode 13h (320x200 linear 256-color mode)
! mode X (320x240 planar 256 color mode)

|-
| Mode Control
| 0x3C0
| 0x10
| 0x0C
| 0x01
| 0x41
| 0x41

|-
| Overscan Register
| 0x3C0
| 0x11
| 0x00
| 0x00
| 0x00
| 0x00

|-
| Color Plane Enable
| 0x3C0
| 0x12
| 0x0F
| 0x0F
| 0x0F
| 0x0F

|-
| Horizontal Panning
| 0x3C0
| 0x13
| 0x08
| 0x00
| 0x00
| 0x00

|-
| Color Select
| 0x3C0
| 0x14
| 0x00
| 0x00
| 0x00
| 0x00

|-
| Miscellaneous Output Register
| 0x3C2
| N/A
| 0x67
| 0xE3
| 0x63
| 0xE3

|-
| Clock Mode Register
| 0x3C4
| 0x01
| 0x00
| 0x01
| 0x01
| 0x01

|-
| Character select
| 0x3C4
| 0x03
| 0x00
| 0x00
| 0x00
| 0x00

|-
| Memory Mode Register
| 0x3C4
| 0x04
| 0x07
| 0x02
| 0x0E
| 0x06

|-
| Mode Register
| 0x3CE
| 0x05
| 0x10
| 0x00
| 0x40
| 0x40

|-
| Miscellaneous Register
| 0x3CE
| 0x06
| 0x0E
| 0x05
| 0x05
| 0x05

|-
| Horizontal Total
| 0x3D4
| 0x00
| 0x5F
| 0x5F
| 0x5F
| 0x5F

|-
| Horizontal Display Enable End
| 0x3D4
| 0x01
| 0x4F
| 0x4F
| 0x4F
| 0x4F

|-
| Horizontal Blank Start
| 0x3D4
| 0x02
| 0x50
| 0x50
| 0x50
| 0x50

|-
| Horizontal Blank End
| 0x3D4
| 0x03
| 0x82
| 0x82
| 0x82
| 0x82

|-
| Horizontal Retrace Start
| 0x3D4
| 0x04
| 0x55
| 0x54
| 0x54
| 0x54

|-
| Horizontal Retrace End
| 0x3D4
| 0x05
| 0x81
| 0x80
| 0x80
| 0x80

|-
| Vertical Total
| 0x3D4
| 0x06
| 0xBF
| 0x0B
| 0xBF
| 0x0D

|-
| Overflow Register
| 0x3D4
| 0x07 
| 0x1F
| 0x3E
| 0x1F
| 0x3E

|-
| Preset row scan
| 0x3D4
| 0x08
| 0x00
| 0x00
| 0x00
| 0x00

|-
| Maximum Scan Line
| 0x3D4
| 0x09
| 0x4F
| 0x40
| 0x41
| 0x41

|-
| Vertical Retrace Start
| 0x3D4
| 0x10
| 0x9C
| 0xEA
| 0x9C
| 0xEA

|-
| Vertical Retrace End
| 0x3D4
| 0x11
| 0x8E
| 0x8C
| 0x8E
| 0xAC

|-
| Vertical Display Enable End
| 0x3D4
| 0x12
| 0x8F
| 0xDF
| 0x8F
| 0xDF

|-
| Logical Width
| 0x3D4
| 0x13
| 0x28
| 0x28
| 0x28
| 0x28

|-
| Underline Location
| 0x3D4
| 0x14
| 0x1F
| 0x00
| 0x40
| 0x00

|-
| Vertical Blank Start
| 0x3D4
| 0x15
| 0x96
| 0xE7
| 0x96
| 0xE7

|-
| Vertical Blank End
| 0x3D4
| 0x16
| 0xB9
| 0x04
| 0xB9
| 0x06

|-
| Mode Control
| 0x3D4
| 0x17
| 0xA3
| 0xE3
| 0xA3
| 0xE3

|}

== See Also ==

=== External Links ===

* https://web.archive.org/web/20251231201151/http://tinyvga.com/vga-timing — VGA Signal Timing
* http://www.osdever.net/FreeVGA/vga/vga.htm — includes some things not explained here

[[Category:VGA]]
[[Category:Video]]
[[Category:Hardware]]
````

<a id="w001698"></a>

## W001698 — Garbage collection

Bron-ID: W001698
Namespace: Main (0)
Revisie: 19321 | Datum: 2016-06-02T20:59:53Z
Bron: https://wiki.osdev.org/Garbage_collection
Revisie-URL: https://wiki.osdev.org/index.php?title=Garbage_collection&oldid=19321

````text
'''Garbage Collection''' ('''GC''') is a [[memory management]] technique frequently used in high-level languages that allows the programmer not to worry about when memory areas should be returned to the system. Virtually all the [[object-oriented language]]s introduced after [[C++]] provide some way of garbage collection (including Python, Java, Objective C). It can also be found in LISP or PERL, for instance.

== How does it work? ==

=== Reference counting ===

The most basic implementation of garbage collection uses '''reference counting''': each object is associated with a counter that tells how many other objects refer to it. Say for instance you have a <tt>Disk</tt> object, every time your system needs another reference to that object (for instance because the <tt>DiskPartition</tt> object has a reference to the parent <tt>Disk</tt> object), the reference counter of <tt>Disk</tt> is incremented. Of course, if programmed manually, this is tedious and bugprone (although in C++ at least, the use of [[Smart_Pointer|smart pointer]]s can automate this bookkeeping). Plus GC's solely based on reference counting will be unable to free self-referencing (or circular) structures, meaning that memory leaks are possible.

=== Mark & Sweep ===

A '''mark-sweep garbage collector''' traverses all reachable objects in the heap by following pointers beginning with the "roots", i.e. pointers stored in statically allocated or [[stack]] allocated program variables (and possibly registers as well, depending on the GC implementation). All such reachable objects are marked. A sweep over the entire heap is then performed to restore unmarked objects to a free list, so they can be reallocated.

: ''Drawn from Hans Boehm's texts, see links below.''

=== Copy collectors ===

In contrast, a '''copying collector''' copies reachable objects to another region of memory as they are being traversed. Provided the traversal is done in breadth first order, there is a well-known and simple algorithm for performing this traversal without auxiliary storage or recursion. After such a traversal all surviving objects reside in a contiguous region of memory, and all pointers have been updated to point to the new object locations. The previously used region of memory can then be reused in its entirety. Allocation becomes trivial, since all free space is always contiguous.

During the process, the GC builds an object graph to track the "live" objects so that it can update references to any objects that it moves. This means that with a compacting GC, there is no extra level of indirection required in order to move objects around in memory. It also means that programmers cannot rely on the values of references remaining stable during execution of their program, which is one reason why languages such as Java and C# disallow pointer arithmetic on references to garbage collected objects.

A variant of compacting collectors is the '''generational garbage collector''' where the memory is partitioned among recently-created objects and longer-lived objects. If an object remains present in the "recently-created" heap for ''N'' sweeps of the GC, it is then moved to the "longer-lived" heap. This technique is used by some JVMs and the .NET CLR to reduce the cost of heap-sweeping as the heap grows larger. It is based on the assumption that a long-lived object is less likely to become garbage and thus that the "long-lived objects" heap could be swept with lower frequency.

One of the major advantages of a compacting collector is that it makes allocating from the heap extremely fast and scalable to multiprocessor systems. All you have to do to allocate a new object is to increment a "top-of-heap" pointer and initialize a few fields, which can be done in constant time. A primitive <tt>malloc()</tt> would, in comparison, have to scan the free block list to find a block large enough to satisfy the request, with a worst-case complexity of ''O''(''n'') (''n'' being the number of free blocks), all the while holding a global heap lock &ndash; not a Good Idea<sup>tm</sup> in a multithreading system. (Modern <tt>malloc()</tt> implementations use a number of techniques to reduce the impact of this, and achieve ''O''(1) behaviour for most cases.)

On the other hand, most compacting collectors need to "stop the world" (i.e. suspend all threads) during a collection so that they can safely move objects around. This makes them very unsuitable for use in a real-time system, and that most likely includes OS kernels as well.

== Can I use garbage collection in my OS? ==

There isn't a real agreement on that among the community. Some says you shouldn't even think about it, other says it is fully possible. If this is your very first OS and unless you start your OS with the goal of proving that a garbage collector in kernel land is possible and better than by-hand memory management, then probably you shouldn't mess with garbage collection.

Whatever you chose to do, make sure that your garbage collector implementation is fully tested and stressed in a host environment before moving it into kernel space. And that missing a few garbage items is probably better than collecting live items :P

The "Singularity" project (Microsoft Research, closed-source) and the [http://www.flingos.co.uk FlingOS (educational OS, open-source)] both use GC everywhere including in the kernel, so it can be done. Yet it needs to be done properly and it has important implications on your kernel design:
* Using a single garbage-collected heap for every chunk of memory (both kernel and user) is probably a Bad Thing<sup>tm</sup>. You are most likely to have collected kernel-wide heap, collected process(/thread)-pinned heap(s) and uncollected kernel heap.
* Having the garbage-collector freeze everything to work is unacceptable at kernel level, yet not all GC's require this (concurent mark & sweep being an example).
* Garbage collection is often coupled with strong object typing and important restrictions on pointers arithmetic which might not be practical in C/C++ ... still under discussion.

== References ==

=== Forum ===

* [http://www.osdev.org/phpBB2/viewtopic.php?t=11400 "Newbie Memory Management Question"]
* [http://www.osdev.org/phpBB2/viewtopic.php?t=11332 "Digital Mars C/D compilers"]

=== Websites ===

* [[wikipedia:Garbage collection (computer science)|Wikipedia on Garbage collection]]
* http://www-128.ibm.com/developerworks/library/l-memory/ - IBM article about memory management
* ftp://ftp.research.microsoft.com/pub/tr/TR-2005-135.pdf - Microsoft Singularity Technical Report
* [http://msdn.microsoft.com/library/default.asp?url=/msdnmag/issues/1100/gci/toc.asp "GC in the .NET Framework Part I"]
* [http://msdn.microsoft.com/msdnmag/issues/1200/GCI2/default.aspx "GC in the .NET Framework Part II"]
* http://www.hpl.hp.com/personal/Hans_Boehm/gc/

[[Category:Memory management]]
````

<a id="w001704"></a>

## W001704 — Stack

Bron-ID: W001704
Namespace: Main (0)
Revisie: 26459 | Datum: 2021-10-04T21:23:30Z
Bron: https://wiki.osdev.org/Stack
Revisie-URL: https://wiki.osdev.org/index.php?title=Stack&oldid=26459

````text
:''Stack can also refer to the TCP/IP stack in [[:Category:Networking|Networking]]. This article discuss the datastructure and stacks used in architectures.''

[[Image:Stack.png|thumb|right|A normal stack, that grows upwards.]]

A '''stack''' is a datastructure. You can push and pop elements to and from it respectively. Unlike a [[FIFO]] (first in, first out) however, the popped elements from a stack are the elements that you pushed last. Because of this, a stack is also termed LIFO (last in, first out) or FILO (first in, last out).

In the [[:Category:X86|X86 architecture]], and many others, there is one stack used for code execution. It is used to store return pointers when calling routines, but you can also store temporary data and local variables on it.


== Stack theory ==

Many languages and architectures have a stack at their disposal. When return values are stored on it, the concept of stack frames emerges. A stack is divided into a number of stack frames. Each stack frame contains the local/temporary data for the routine, parameters, and a return value to the former routine (the caller).

=== Stack example on the X86 architecture ===

On the X86 architecture the stack grows downwards. The stack frames have a certain structure regarding the calling convention. The CDECL calling convention is the most widely used. It is most likely used by your compiler. Two registers are used:

* '''<tt>ESP</tt>:''' Extended Stack Pointer. 32 bit value containing the ''top-of-stack'' address (more accurately the ``bottom-of-stack`` on X86!)
* '''<tt>EBP</tt>:''' Extended Base Pointer. 32 bit value defining the current stack frame, when using CDECL calling convention. It points at the current local data. It can also access the routine parameters.

Take care when implementing your kernel. If you use [[segmentation]], the <tt>DS</tt> segment should be configured to have its base at the same address as SS does. Otherwise you may run into problems when passing pointers to local variables into functions, because <tt>normal GPRs</tt> can't access the stack the way you might think.

Here is an example stack. The elements are 4 byte words in protected mode:

 Memory address:   Stack elements:
 
                +----------------------------+
  0x105000      | Parameter 1 for routine 1  | \
                +----------------------------+  |
  0x104FFC      | First callers return addr. |   >  Stack frame 1
                +----------------------------+  |
  0x104FF8      | First callers EBP          | /
                +----------------------------+
  0x104FF4   +->| Parameter 2 for routine 2  | \  <-- Routine 1's EBP
             |  +----------------------------+  |
  0x104FF0   |  | Parameter 1 for routine 2  |  |
             |  +----------------------------+  |
  0x104FEC   |  | Return address, routine 1  |  |
             |  +----------------------------+  |
  0x104FE8   +--| EBP value for routine 1    |   >  Stack frame 2
                +----------------------------+  |
  0x104FE4   +->| Local data                 |  | <-- Routine 2's EBP
             |  +----------------------------+  |
  0x104FE0   |  | Local data                 |  |
             |  +----------------------------+  |
  0x104FDC   |  | Local data                 | /
             |  +----------------------------+
  0x104FD8   |  | Parameter 1 for routine 3  | \
             |  +----------------------------+  |
  0x104FD4   |  | Return address, routine 2  |  |
             |  +----------------------------+   >  Stack frame 3
  0x104FD0   +--| EBP value for routine 2    |  |
                +----------------------------+  |
  0x104FCC   +->| Local data                 | /  <-- Routine 3's EBP
             |  +----------------------------+
  0x104FC8   |  | Return address, routine 3  | \
             |  +----------------------------+  |
  0x104FC4   +--| EBP value for routine 3    |  |
                +----------------------------+   >  Stack frame 4
  0x104FC0      | Local data                 |  | <-- Current EBP
                +----------------------------+  |
  0x104FBC      | Local data                 | /
                +----------------------------+
  0x104FB8      |                            |    <-- Current ESP
                 \/\/\/\/\/\/\/\/\/\/\/\/\/\/

The CDECL calling convention is described here:

; Caller's responsibilities
* Push parameters in reverse order (last parameter pushed first)
* Perform the call
* Pop the parameters, use them, or simply increment ESP to remove them (stack clearing)
* The return value is stored in EAX

; Callee's responsibilities (callee is the routine being called)
* Store caller's EBP on the stack
* Save current ESP in EBP
* Code, storing local data on the stack
* For a fast exit load the old ESP from EBP, else pop local data elements
* Pop the old EBP and return &ndash; store return value in EAX

It looks like this in assembly (NASM):

 SECTION .text
 
 caller:
     
     ; ...
     
     ; Caller responsibilities:
     PUSH  3         ; push the parameters in reverse order
     PUSH  2
     CALL  callee    ; perform the call
     ADD   ESP, 8    ; stack cleaning (remove the 2 words)
     
     ; ... Use the return value in EAX ...
     
     
 callee:
     
     ; Callee responsibilities:
     PUSH  EBP       ; store caller's EBP
     MOV   EBP, ESP  ; save current stack pointer in EBP
     
     ; ... Code, store return value in EAX ...
     
     ; Callee responsibilities:
     MOV   ESP, EBP  ; remove an unknown number of local data elements
     POP   EBP       ; restore caller's EBP
     RET             ; return

The GCC compiler does all this automatically, but if you have to call C/C++ methods from assembly or reverse, you have to know the convention. Now look at one stack frame (the callee's):

 +-------------------------+
 | Parameter 3             |
 +-------------------------+
 | Parameter 2             |
 +-------------------------+
 | Parameter 1             |
 +-------------------------+
 | Caller's return address |
 +-------------------------+
 | Caller's EBP value      |
 +-------------------------+
 | Local variable          | <-- Current EBP
 +-------------------------+
 | Local variable          |
 +-------------------------+
 | Local variable          |
 +-------------------------+
 | Temporary data          |
 +-------------------------+
 | Temporary data          |
 +-------------------------+
 |                         | <-- Current ESP
 +-------------------------+

Using <tt>EBP</tt> the callee can access both parameters and local variables:

 MOV  EAX, <nowiki>[[EBP + 12]]</nowiki>  ; Load parameter 1 into EAX
 
 MOV  EAX, <nowiki>[[EBP + 16]]</nowiki>  ; Load parameter 2
 
 MOV  EAX, <nowiki>[[EBP + 4 * EBX + 12]]</nowiki>  ; Load parameter EBX (0-indexed)
 
 MOV  EAX, <nowiki>[[EBP]]</nowiki>       ; Load local variable 1
 
 MOV  EAX, <nowiki>[[EBP - 4]]</nowiki>   ; Load local variable 2

There are other calling conventions for X86. To name a few: Pascal calling convention, fastcall convention, stdcall. Read more on Wikipedia, see the links below.

=== Set up the stack ===

When creating a kernel you have to manually set up the stack. It can be done by the [[bootloader]], but it is good to know how anyway.

If you go from real mode to protected mode, you have to set up the stack as well. This is because the <tt>SS</tt> segment might change, so <tt>ESP</tt> in protected mode does not point to the same location that <tt>SP</tt> in real mode did. If you switch a lot between real mode and protected mode, it can be nice for them to share the stack. You have to find a smart solution for that yourself. It can be done.

In protected mode you set up the stack by just moving a new pointer value into the <tt>ESP</tt> register:

 MOV  ESP, 0x105000  ; Set the stack pointer

Remember that it grows downwards. You might allocate space for it in the kernel's <tt>.BSS</tt> section if it contains one:

 SECTION .text
 
 set_up_stack:
     
     MOV  ESP, stack_end  ; Set the stack pointer
 
 SECTION .bss
 
 stack_begin:
     RESB 4096  ; Reserve 4 KiB stack space
 stack_end:

If your kernel is booted by a [[Multiboot]]-compliant bootloader, like [[GRUB]], you are provided a memory map. You can set up the stack by looking for free memory chunks of the appropriate size. You just have to ensure that you don't overwrite any important data or code when setting the stack pointer.

=== Security ===

The stack is easy to use, but it has one problem. There is no "end," so it is vulnerable to a variation of the buffer overflow attack. The attacker pushes more elements than the stack is able to contain, so elements are pushed outside the stack memory, overwriting code, which the attacker can then execute.

In X86 protected mode it can be solved by allocating a [[GDT|GDT descriptor]] solely for the stack, which defines its boundaries.

== Stack trace ==

When debugging, a stack trace is often shown and can be helpful. [[Stack Trace]] describes how this can be done and provides sample code for X86 CDECL using the stack layout above.

== Unwinding the stack ==

Unwinding the stack is complex. It is done when using exceptions, like in [[C++]]. It is performed when an exception is thrown. The purpose of unwinding the stack is to call the destructor of local objects of the stack frames and to remove stack frames until an appropriate landing pad is found. The landing pad is the try..catch block in C++ or Java. The catch block has to match the exception, i.e. a RuntimeException object can't be caught as a String object.

The unwinding algorithm depends on the architecture. Normally this algorithm is provided in the language runtime library. When using GCC and C++ it is defined in the libsupc++ library linked with your application. However this doesn't happen when creating a kernel. The libsupc++ library is also too bloated to use in kernel space.

== See Also ==

=== Articles ===
[[Stack Trace]] - Trace the called functions from the stack

=== Threads ===

=== External Links ===

* [[wikipedia:x86 calling conventions| x86 calling conventions]] on Wikipedia.
* [[wikipedia:Stack (data structure)| Stack (data structure)]] on Wikipedia.

[[Category:OS theory]]
````

<a id="w001705"></a>

## W001705 — File:VGA gc addressdecode.gif

Bron-ID: W001705
Namespace: File (6)
Revisie: 2505 | Datum: 2006-12-20T23:07:31Z
Bron: https://wiki.osdev.org/File:VGA_gc_addressdecode.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AVGA_gc_addressdecode.gif&oldid=2505

````text
VGA Graphics Controller datapaths
````

<a id="w001706"></a>

## W001706 — Inline Assembly

Bron-ID: W001706
Namespace: Main (0)
Revisie: 30389 | Datum: 2026-03-23T13:58:23Z
Bron: https://wiki.osdev.org/Inline_Assembly
Revisie-URL: https://wiki.osdev.org/index.php?title=Inline_Assembly&oldid=30389

````text
The idea behind '''Inline Assembly''' is to embed assembler instructions in your C/C++ code, using the <tt>asm</tt> keyword, when there's no option but to use [[Assembly]] language.

== Overview ==
Sometimes, even though C/C++ is your language of choice, you '''need''' to use some assembler code in your operating system. Be it because of extreme optimization needs or because the code you're implementing is highly hardware-specific (like, say, outputting data through a port), the result is the same: there's no way around it. You must use assembly.

One of the options you have is writing an assembly function and calling it, however there can be times when even the "call" overhead is too much for you. In that case, what you need is inline assembly, which means inserting arbitrary assembly snippets in the middle of your code, using the <tt>asm()</tt> keyword. The way this keyword works is compiler-specific. This article describes the way it works in GCC since it is by far the most used compiler in the OS world.

== Syntax ==
This is the syntax for using the <tt>asm()</tt> keyword in your C/C++ code:

<syntaxhighlight lang="c">
asm ( assembler template
    : output operands                   (optional)
    : input operands                    (optional)
    : clobbered registers list          (optional)
    );
</syntaxhighlight>

Assembler template is basically [[GAS]]-compatible code, except when you have constraints, in which case register names must start with %% instead of %. This means that the following two lines of code will both move the contents of the <tt>eax</tt> register into <tt>ebx</tt>:

<syntaxhighlight lang="c">
asm ("movl %eax, %ebx");
asm ("movl %%eax, %%ebx" : );
</syntaxhighlight>

Now, you may wonder why this %% comes in. This is where an interesting feature of inline assembly comes in: you can make use of some of your C variables in your assembly code. And since, in order to make implementation of this mechanism simpler, GCC names these variables %0, %1, and so on in your assembly code, starting from the first variable mentioned in the input/output operand sections. You're required to use this %% syntax in order to help GCC differentiate between registers and parameters.

How exactly operands work will be explained in more details in later sections. For now, it is sufficient to say that if you write something like that:

<syntaxhighlight lang="c">
int a=10, b;
asm ("movl %1, %%eax; 
      movl %%eax, %0;"
     :"=r"(b)        /* output */
     :"r"(a)         /* input */
     :"%eax"         /* clobbered register */
     );
</syntaxhighlight>

then you've managed to copy the value of "a" into "b" using assembly code, effectively using some C variables in your assembly code. Congratulations!

The last "clobbered register" section is used in order to tell GCC that your code is using some of the processor's registers, and that it should move any active data from the running program out of this register before executing the asm snippet. In the example above, we move <tt>a</tt> to eax in the first instruction, effectively erasing its content, so we need to ask GCC to clear this register from unsaved data before operation.

=== Assembler Template ===
The Assembler Template defines the assembler instructions to inline. The default is to use AT&T syntax here. If you want to use Intel syntax, <tt>-masm=intel</tt> should be specified as a command-line option.

As an example, to halt the CPU, you just have to use the following command:

<syntaxhighlight lang="c">
asm( "hlt" );
</syntaxhighlight>

=== Output Operands ===
The Output Operands section is used in order to tell the compiler / assembler how it should handle C variables used to store some output from the ASM code. The Output Operands are a list of pairs, each operand consisting of a string literal, known as "constraint", stating where the C variable should be mapped (registers are generally used for optimal performance), and a C variable to map to (in parentheses).

In the constraint, 'a' refers to EAX, 'b' to EBX, 'c' to ECX, 'd' to EDX, 'S' to ESI, and 'D' to EDI (read the GCC manual for a full list), assuming that you are coding for the IA32 architecture. An equation sign indicates that your assembly code does not care about the initial value of the mapped variable (which allows some optimization). With all that in mind, it's now pretty clear that the following code sets EAX = 0.

<syntaxhighlight lang="c">
int EAX;
asm( "movl $0, %0"
   : "=a" (EAX)
    );
</syntaxhighlight>

Notice that the compiler enumerates the operand starting with %0, and that you don't have to add a register to the clobbered register list if it's used to store an output operand. GCC is smart enough to figure out what to do all by itself.

Starting with GCC 3.1, you can use more readable labels instead of the error-prone enumeration:

<syntaxhighlight lang="c">
int current_task;
asm( "str %[output]"
   : [output] "=r" (current_task)
    );
</syntaxhighlight>

These labels are in a namespace of their own, and will not collide with any C identifiers. The same can be done for input operands, too.

=== Input Operands ===
While the Output Operands are generally used for... well... output, the Input Operands allows to parametrize the ASM code; i.e., passing read-only parameters from C code to ASM block. Again, string literals are used to specify the details.

If you want to move some value to EAX, you can do it the following way (even though it would certainly be pretty useless to do so instead of directly mapping the value to EAX):

<syntaxhighlight lang="c">
int randomness = 4;
asm( "movl %0, %%eax"
   :
   : "b" (randomness)
   : "eax"
    );
</syntaxhighlight>

Note that GCC will always assume that input operands are read-only (unchanged). The correct thing to do when input operands are written to is to list them as outputs, but without using the equation sign because this time their original value matters. Here is a simple example:
<syntaxhighlight lang="c">
asm("mov %%eax,%%ebx": : "a" (amount)); // useless but it gets the idea
</syntaxhighlight>
Eax will contain "amount" and be moved into ebx.

=== Clobbered Registers List ===
It is important to remember one thing: ''The C/C++ compiler knows nothing about Assembler''. For the compiler, the asm statement is opaque, and if you did not specify any output, it might even come to the conclusion that it's a no-op and optimize it away. Some third-party docs indicate that using asm volatile will cause the keyword to not be moved. However, according to the GCC documentation, ''The volatile keyword indicates that the instruction has important side-effects. GCC will not delete a volatile asm if it is reachable.'', which only indicates that it will not be deleted (i.e. whether it may still be moved is an unanswered question). An approach that should work is to use asm (volatile) and put '''memory''' in the clobber registers, like so:

<syntaxhighlight lang="c">
__asm__("cli": : :"memory"); // Will cause the statement not to be moved, but it may be optimized away.
__asm__ __volatile__("cli": : :"memory"); // Will cause the statement not to be moved nor optimized away.
</syntaxhighlight>

Since the compiler uses CPU registers for internal optimization of your C/C++ variables, and doesn't know about ASM opcodes, you have to warn it about any registers that might get clobbered as a side effect, so the compiler can save their contents before making your ASM call.

The Clobbered Registers List is a comma-separated list of register names, as string literals.

=== Wildcards: How you can let the compiler choose ===
You don't need to tell the compiler which specific register it should use in each operation, and in general, except you have good reasons to prefer one register specifically, you should better let the compiler decide for you.

Forcing to use EAX over any other register, for instance, may force the compiler to issue code that will save what was previously in eax in some other register or may introduce unwanted dependencies between operations (pipeline optimization broken)

The 'wildcards' constraints allows you to give more freedom to GCC when it comes to input/output mapping:
{| {{wikitable}}
|-
| The "g" constraint : <syntaxhighlight lang="c">"movl $0, %0" : "=g" (x)</syntaxhighlight>
| x can be whatever the compiler prefers: a register, a memory reference. It could even be a literal constant in another context.
|-
| The "r" constraint : <syntaxhighlight lang="c">"movl %%es, %0" : "=r" (x)</syntaxhighlight>
| you want x to go through a register. If x wasn't optimized as a register, the compiler will then move it to the place it should be. This means that <code>"movl %0, %%es" : : "r" (0x38)</code> is enough to load a segment register.
|-
| The "N" constraint : <syntaxhighlight lang="c">"outl %0, %1" : : "a" (0xFE), "N" (0x21)</syntaxhighlight>
| tells the value '0x21' can be used as a constant in the out or in operation if ranging from 0 to 255
|}
There are of course a lot more constraints you can put on the operand selection, machine-dependent or not, which are listed in GCC's manual (see [http://gcc.gnu.org/onlinedocs/gcc-4.4.4/gcc/Simple-Constraints.html#Simple-Constraints], [http://gcc.gnu.org/onlinedocs/gcc-4.4.4/gcc/Modifiers.html#Modifiers], [http://gcc.gnu.org/onlinedocs/gcc-4.4.4/gcc/Multi_002dAlternative.html#Multi_002dAlternative], and [http://gcc.gnu.org/onlinedocs/gcc-4.4.4/gcc/Machine-Constraints.html#Machine-Constraints]).

== Using C99 ==

<tt>asm</tt> is not a keyword when using <tt>gcc -std=c99</tt>. Simply use <tt>gcc -std=gnu99</tt> to use C99 with GNU extensions. Alternatively, you can use <tt>__asm__</tt> as an alternate keyword that works even when the compiler strictly adheres to the standard.

== Assigning Labels ==
It is possible to assign so-called ASM labels to C/C++ keywords. You can do this by using the <tt>asm</tt> command on variable definitions, as seen in this example:

<syntaxhighlight lang="c">
int some_obscure_name asm("param") = 5; // "param" will be accessible in inline Assembly.

void foo()
{
   asm("mov param, %%eax");
}
</syntaxhighlight>

Here's an example of how you can access these variables if you don't explicitly state a name:

<syntaxhighlight lang="c">
int some_obscure_name = 5;

void foo()
{
   asm("mov some_obscure_name, %%eax");
}
</syntaxhighlight>

Note that you might also be obliged to use '''_some_obscure_name''' (with a leading underscore), depending on your linkage options.

== asm goto ==
Before GCC 4.5, jumping across inline assembly blocks is not supported. The compiler has no way of keeping track of what's going on,
so incorrect code is almost guaranteed to be generated.
<br>You might have been told that "gotos are evil". If you believe that is so, then asm gotos are your worst nightmare coming true.
However, they do offer some interesting code optimization options.

asm goto's are not well documented, but their syntax is as follows:
<syntaxhighlight lang="c">
 asm goto( "jmp %l[labelname]" : /* no outputs */ : /* inputs */ : "memory" /* clobbers */ : labelname /* any labels used */ );
</syntaxhighlight>

One example where this can be useful, is the CMPXCHG instruction (see [http://en.wikipedia.org/wiki/Compare-and-swap Compare and Swap]), which the Linux kernel source code defines as follows:
<syntaxhighlight lang="c">
/* TODO: You should use modern GCC atomic instruction builtins instead of this. */
#include <stdint.h>
#define cmpxchg( ptr, _old, _new ) { \
  volatile uint32_t *__ptr = (volatile uint32_t *)(ptr);   \
  uint32_t __ret;                                     \
  asm volatile( "lock; cmpxchgl %2,%1"           \
    : "=a" (__ret), "+m" (*__ptr)                \
    : "r" (_new), "0" (_old)                     \
    : "memory");				 \
  );                                             \
  __ret;                                         \
}
</syntaxhighlight>

In addition to returning the current value in EAX, CMPXCHG sets the zero flag (Z) when successful. Without asm gotos, your code will have to check the returned value; 
this CMP instruction can be avoided as follows:

<syntaxhighlight lang="c">
/* TODO: You should use modern GCC atomic instruction builtins instead of this. */
// Works for both 32 and 64 bit
#include <stdint.h>
#define cmpxchg( ptr, _old, _new, fail_label ) { \
  volatile uint32_t *__ptr = (volatile uint32_t *)(ptr);   \
  asm goto( "lock; cmpxchg %1,%0 \t\n"           \
    "jnz %l[" #fail_label "] \t\n"               \
    : /* empty */                                \
    : "m" (*__ptr), "r" (_new), "a" (_old)       \
    : "memory", "cc"                             \
    : fail_label );                              \
}
</syntaxhighlight>

This new macro could then be used as follows:
<syntaxhighlight lang="c">

struct Item {
  volatile struct Item* next;
};

volatile struct Item *head;

void addItem( struct Item *i ) {
  volatile struct Item *oldHead;
again:
  oldHead = head;
  i->next = oldHead;
  cmpxchg( &head, oldHead, i, again );
}

</syntaxhighlight>

== Intel Syntax ==
You can let GCC use intel syntax by enabling it in inline Assembly, like so:

<syntaxhighlight lang="c">
asm(".intel_syntax noprefix");
asm("mov eax, ebx");
</syntaxhighlight>

Similarly, you can switch back to AT&T syntax by using the following snippet:

<syntaxhighlight lang="c">
asm(".att_syntax prefix");
asm("mov %ebx, %eax");
</syntaxhighlight>

This way you can combine Intel syntax and AT&T syntax inline Assembly. Note that once you trigger one of these syntax types, everything below the command in the source file will be assembled using this syntax, so don't forget to switch back when necessary, or you might get lots of compile errors!

There is also a command-line option <tt>-masm=intel</tt> to globally trigger Intel syntax.

GCC also supports multiple assembler dialects by wrapping text in braces separated by a vertical bar. The two examples above can be combined as follows, where the first version will be emitted when <tt>-masm=att</tt> is used, or no <tt>-masm</tt> is provided, and the second will be used when <tt>-masm=intel</tt> is provided.

<syntaxhighlight lang="c">
asm("{mov %ebx, %eax|mov eax, ebx}");
</syntaxhighlight>

== See Also ==

=== Articles ===
* [[Inline Assembly/Examples]] - useful and commonly used functions

=== Forum Threads ===
* [http://forum.osdev.org/viewtopic.php?f=11&t=24168&p=196655&hilit=asm+volatile+moved asm volatile being moved]

=== External ===
* [http://gcc.gnu.org/onlinedocs/ GCC Manuals]
* [http://web.archive.org/web/20041210030000/http://www-106.ibm.com/developerworks/library/l-ia.html Inline assembly for x86 in Linux (by IBM)]
* [http://msdn.microsoft.com/en-us/library/26td21ds(VS.80).aspx Visual C++ Compiler Intrinsics]

[[Category:Assembly]]
[[de:Inline-Assembler_mit_GCC]]
````

<a id="w001707"></a>

## W001707 — Inline Assembly/Examples

Bron-ID: W001707
Namespace: Main (0)
Revisie: 28596 | Datum: 2024-02-01T10:49:24Z
Bron: https://wiki.osdev.org/Inline_Assembly/Examples
Revisie-URL: https://wiki.osdev.org/index.php?title=Inline_Assembly%2FExamples&oldid=28596

````text
The following is a collection of [[Inline Assembly]] functions so common that they should be useful to most OS developers using [[GCC]].  Other compilers may have intrinsic alternatives (see references). Notice how these functions are implemented using GNU extensions to the [[C]] language and that particular keywords may cause you trouble if you disable GNU extensions. You can still use the disabled keywords such as <tt>asm</tt> if you instead use the alternate keywords in the reserved namespace such as <tt>__asm__</tt>. Be wary of getting inline assembly just right: The compiler doesn't understand the assembly it emits and can potentially cause rare nasty bugs if you lie to the compiler.

'''WARNING!'''

Be aware, <tt>asm("");</tt> <small>(no constaints, known as [https://gcc.gnu.org/onlinedocs/gcc/Basic-Asm.html basic <tt>asm</tt>])</small> and <tt>asm("":);</tt> <small>(empty constraints, a form of [https://gcc.gnu.org/onlinedocs/gcc/Extended-Asm.html extended <tt>asm</tt>])</small> are not the same! Among other differences, in basic <tt>asm</tt> you must prefix registers with a single percentage sign '%', in extended <tt>asm</tt> (even if the list of constraints is empty) you must use '%%' as a prefix in the assembly template string. If you get gcc error messages like "Error: bad register name '%%eax'", then you should insert a colon (':') before the closing parenthesis. Extended <tt>asm</tt> is generally preferred, though there are [https://gcc.gnu.org/onlinedocs/gcc/Basic-Asm.html#Remarks cases where basic <tt>asm</tt> is necessary ]. Also note that this register prefix only applies to AT&T syntax assembly, gcc's default.


== Memory access ==

=== FAR_PEEKx ===

Read a 8/16/32-bit value at a given memory location using another segment than the default C data segment. Unfortunately there is no constraint for manipulating segment registers directly, so issuing the <tt>mov <reg>, <segmentreg></tt> manually is required.

<source lang="c">
static inline uint32_t farpeekl(uint16_t sel, void* off)
{
    uint32_t ret;
    asm ( "push %%fs\n\t"
          "mov  %1, %%fs\n\t"
          "mov  %%fs:(%2), %0\n\t"
          "pop  %%fs"
          : "=r"(ret) : "g"(sel), "r"(off) );
    return ret;
}
</source>

=== FAR_POKEx ===

Write a 8/16/32-bit value to a segment:offset address too. Note that much like in farpeek, this version of farpoke saves and restore the segment register used for the access.

<source lang="c">
static inline void farpokeb(uint16_t sel, void* off, uint8_t v)
{
    asm ( "push %%fs\n\t"
          "mov  %0, %%fs\n\t"
          "movb %2, %%fs:(%1)\n\t"
          "pop %%fs"
          : : "g"(sel), "r"(off), "r"(v) );
    /* TODO: Should "memory" be in the clobber list here? */
}
</source>

== I/O access ==

=== OUTx ===

Sends a 8/16/32-bit value on a I/O location. Traditional names are <tt>outb</tt>, <tt>outw</tt> and <tt>outl</tt> respectively. The <tt>a</tt> modifier enforces val to be placed in the eax register before the asm command is issued and <tt>Nd</tt> allows for one-byte constant values to be assembled as constants, freeing the edx register for other cases.  

<source lang="c">
static inline void outb(uint16_t port, uint8_t val)
{
    __asm__ volatile ( "outb %b0, %w1" : : "a"(val), "Nd"(port) : "memory");
    /* There's an outb %al, $imm8 encoding, for compile-time constant port numbers that fit in 8b. (N constraint).
     * Wider immediate constants would be truncated at assemble-time (e.g. "i" constraint).
     * The  outb  %al, %dx  encoding is the only option for all other cases.
     * %1 expands to %dx because  port  is a uint16_t.  %w1 could be used if we had the port number a wider C type */
}
</source>

=== INx ===

Receives a 8/16/32-bit value from an I/O location. Traditional names are <tt>inb</tt>, <tt>inw</tt> and <tt>inl</tt> respectively.

<source lang="c">
static inline uint8_t inb(uint16_t port)
{
    uint8_t ret;
    __asm__ volatile ( "inb %w1, %b0"
                   : "=a"(ret)
                   : "Nd"(port)
                   : "memory");
    return ret;
}
</source>

=== IO_WAIT ===

Wait a very small amount of time (1 to 4 microseconds, generally). Useful for implementing a small delay for PIC remapping on old hardware or generally as a simple but imprecise wait.

You can do an IO operation on any unused port: the Linux kernel by default uses port 0x80, which is often used during POST to log information on the motherboard's hex display but almost always unused after boot.

<source lang="c">
static inline void io_wait(void)
{
    outb(0x80, 0);
}
</source>

== Interrupt-related functions ==

=== Are interrupts enabled? ===

Returns a <tt>true</tt> boolean value if IRQs are enabled for the CPU.

<source lang="c">
static inline bool are_interrupts_enabled()
{
    unsigned long flags;
    asm volatile ( "pushf\n\t"
                   "pop %0"
                   : "=g"(flags) );
    return flags & (1 << 9);
}
</source>

=== Push/pop interrupt flag ===

Sometimes it is helpful to disable interrupts and then only re-enable them if they were disabled before. While the above function can be used for that, the below functions do it the same way regardless of the setting of the state of the IF:

<source lang="c">
static inline unsigned long save_irqdisable(void)
{
    unsigned long flags;
    asm volatile ("pushf\n\tcli\n\tpop %0" : "=r"(flags) : : "memory");
    return flags;
}

static inline void irqrestore(unsigned long flags)
{
    asm ("push %0\n\tpopf" : : "rm"(flags) : "memory","cc");
}

static void intended_usage(void)
{
    unsigned long f = save_irqdisable();
    do_whatever_without_irqs();
    irqrestore(f);
}
</source>

Memory clobber to force ordering, cc clobber because all condition codes are overwritten in the second example. No volatile in the second case since it has no outputs and is thus always volatile.

=== LIDT ===

Define a new interrupt table.

<source lang="c">
static inline void lidt(void* base, uint16_t size)
{
    // This function works in 32 and 64bit mode
    struct {
        uint16_t length;
        void*    base;
    } __attribute__((packed)) IDTR = { size, base };

    asm ( "lidt %0" : : "m"(IDTR) );  // let the compiler choose an addressing mode
}
</source>

== CPU-related functions ==

=== CPUID ===

Request for CPU identification. See [[CPUID]] for more information.

''Note'': [[GCC]] has a <tt>&lt;cpuid.h&gt;</tt> header you should use instead of this.

<source lang="c">
static inline void cpuid(int code, uint32_t* a, uint32_t* d)
{
    asm volatile ( "cpuid" : "=a"(*a), "=d"(*d) : "0"(code) : "ebx", "ecx" );
}
</source>

=== RDTSC ===

Read the current value of the CPU's time-stamp counter and store into EDX:EAX. The time-stamp counter contains the amount of clock ticks that have elapsed since the last CPU reset. The value is stored in a 64-bit MSR and is incremented after each clock cycle.

<source lang="c">
static inline uint64_t rdtsc()
{
    uint64_t ret;
    asm volatile ( "rdtsc" : "=A"(ret) );
    return ret;
}
</source>

This can be used to find out how much time it takes to do certain functions, very useful for testing/benchmarking /etc. Note: This is only an approximation.

On [[X86-64|x86_64]], the "A" constraint expects to write into the "rdx:rax" registers instead of "edx:eax". So GCC can in fact optimize the above code by not setting RDX at all. You instead need to do it manually with bitshifting:

<source lang="c">
uint64_t rdtsc(void)
{
    uint32_t low, high;
    asm volatile("rdtsc":"=a"(low),"=d"(high));
    return ((uint64_t)high << 32) | low;
}
</source>

Read [https://gcc.gnu.org/onlinedocs/gcc/Machine-Constraints.html#Machine-Constraints GCC Inline Assembly Machine Constraints] for more details.

=== READ_CRx ===

Read the value in a control register.

<source lang="c">
static inline unsigned long read_cr0(void)
{
    unsigned long val;
    asm volatile ( "mov %%cr0, %0" : "=r"(val) );
    return val;
}
</source>

=== INVLPG ===

Invalidates the [[TLB]] (Translation Lookaside Buffer) for one specific virtual address. The next memory reference for the page will be forced to re-read PDE and PTE from main memory. Must be issued every time you update one of those tables. The <tt>m</tt> pointer points to a logical address, not a physical or virtual one: an offset for your ds segment.

<source lang="c">
static inline void invlpg(void* m)
{
    /* Clobber memory to avoid optimizer re-ordering access before invlpg, which may cause nasty bugs. */
    asm volatile ( "invlpg (%0)" : : "b"(m) : "memory" );
}
</source>

=== WRMSR ===

Write a 64-bit value to a MSR. The <tt>A</tt> constraint stands for concatenation of registers EAX and EDX.

<source lang="c">
static inline void wrmsr(uint32_t msr_id, uint64_t msr_value)
{
    asm volatile ( "wrmsr" : : "c" (msr_id), "A" (msr_value) );
}
</source>

On x86_64, this needs to be:

<source lang="c">
static inline void wrmsr(uint64_t msr, uint64_t value)
{
    uint32_t low = value & 0xFFFFFFFF;
    uint32_t high = value >> 32;
    asm volatile (
        "wrmsr"
        :
        : "c"(msr), "a"(low), "d"(high)
    );
}
</source>

=== RDMSR ===

Read a 64-bit value from a MSR. The <tt>A</tt> constraint stands for concatenation of registers EAX and EDX.

<source lang="c">
static inline uint64_t rdmsr(uint32_t msr_id)
{
    uint64_t msr_value;
    asm volatile ( "rdmsr" : "=A" (msr_value) : "c" (msr_id) );
    return msr_value;
}
</source>

On x86_64, this needs to be:

<source lang="c">
static inline uint64_t rdmsr(uint64_t msr)
{
    uint32_t low, high;
    asm volatile (
        "rdmsr"
        : "=a"(low), "=d"(high)
        : "c"(msr)
    );
	return ((uint64_t)high << 32) | low;
}
</source>

=== Memory Fence ===

Compiler memory fences can be used to ensure that data is accessed in the specified order by the compiler. A compiler fence is done by specifying memory as a clobber to any inline assembly, telling GCC that memory was supposedly modified, therefore requiring it to stay where it is. This is usually not enough for SMP systems.

<source lang="c">
#define fence() __asm__ volatile ("":::"memory")
</source>

Remember that <tt>volatile</tt> is '''not''' a replacement for a compiler memory fence.

[[Category:Assembly]]
````

<a id="w001708"></a>

## W001708 — Category:Assembly

Bron-ID: W001708
Namespace: Category (14)
Revisie: 8480 | Datum: 2009-07-10T08:16:28Z
Bron: https://wiki.osdev.org/Category:Assembly
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AAssembly&oldid=8480

````text
[[Category:Languages]]
````

<a id="w001709"></a>

## W001709 — File:VGA gc write0.gif

Bron-ID: W001709
Namespace: File (6)
Revisie: 2517 | Datum: 2006-12-21T16:40:31Z
Bron: https://wiki.osdev.org/File:VGA_gc_write0.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AVGA_gc_write0.gif&oldid=2517

````text
Write Mode 0 schematics
````

<a id="w001710"></a>

## W001710 — Category:IPC

Bron-ID: W001710
Namespace: Category (14)
Revisie: 2609 | Datum: 2006-12-29T23:34:34Z
Bron: https://wiki.osdev.org/Category:IPC
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3AIPC&oldid=2609

````text
This is the Category of pages related to inter-process communication

[[Category:Processes and Threads]]
````

<a id="w001711"></a>

## W001711 — Message Passing

Bron-ID: W001711
Namespace: Main (0)
Revisie: 29080 | Datum: 2024-06-07T13:37:33Z
Bron: https://wiki.osdev.org/Message_Passing
Revisie-URL: https://wiki.osdev.org/index.php?title=Message_Passing&oldid=29080

````text
If you consider writing a [[Microkernel]], you should layout how you'll manage the message passing. This page collects considerations on how you can do it.

==Reliable vs Best-Effort==
;reliable
:Messages are guaranteed to be delivered unless the recipient does not exist. On failure, the sender is notified. In order to provide reliability, the message passing system ensures that messages have not been altered or corrupted during transmission and reception. Out-of-order message blocks are reordered and duplicate message blocks are removed. If an error occurs, the system will attempt to retransmit the message automatically. A message's transmission time will tend to be predictable. Reliable delivery of messages, however, usually comes with noticeable overhead.

;best-effort
:Messages may/may not be delivered to the recipient. If a message ''is'' delivered, it may be after a variable delay. Messages could be corrupted, out of order, or duplicated. Also, the sender might not receive any acknowledgment of a successful delivery. This may sound entirely useless, but best-effort delivery is typically simpler to implement than reliable delivery and comes with lower overhead. By implementing a reliable messaging protocol on top of a best-effort delivery protocol, reliable delivery of messages can be achieved (very much like the TCP protocol[reliable] with IP packets[best-effort]). Such a protocol might utilize "ACK" messages and timeouts to support reliable messages.

Reliable delivery is used when data ''must'' be delivered correctly bit-by-bit (as in transferring a file). Best-effort delivery is used when lots of data must be sent with low processing overhead or when it doesn't really matter if a few messages are corrupted or lost (like periodic "I'm alive" messages or when streaming audio/video).

==Synchronous vs Asynchronous==
;synchronous
:Process A delivers the message directly to process B. If process B is not ready to receive at the moment of sending, Process A is suspended and queued upon Process B's 'sending' queue, until Process B is receiving. Using this method, you don't need to care for message queues - but you enqueue processes. Note that if process B is waiting to receive a message from process A, and the message fails to be delivered (due to an error, or some failure in the message delivery system, for example) then process B will block indefinitely.

;asynchronous
:Process A sends the message to Process B. The message is copied to a dedicated region in kernel space and attached to Process B's message queue (lets call it the Post box). The next time, Process B looks into its postbox, it will retrieve the message. You can have processes block for receiving a message or just stroll by and check whether there is something to fetch. One could call this method "store and forward message passing". The main benefit of asynchronous messaging is that it allows a single thread/process to wait for the results of many unrelated "requests" at the same time. However, any erroneous (or malicious) process can repeatedly spam messages in quick succession to other processes, thus tying up system resources in a denial-of-service attack.

We may note that in the synchronous approach, A has the guarantee that B received the message when the deliver call terminates, which makes it especially interesting for local RPC. It should be noted that it can be very difficult to implement standard C library functions without this guarantee.

It isn't a bad idea to wrap some brains around the following problem: What, if Process A expects a message from Process C. It needs a possibility to communicate this to the message routing subsystem. If Process B sends a message to Process A, Process A shall not be woken up for it wants something from Process C. Process C sends a message to Process A. Then and only then shall the message routing subsystem wake up Process A.

What primitives will you need to get synchronous message passing running? Two: request(message) and respond(message).

What primitives will you need to get asynchronous message passing running? Two: send(message) and receive(message).

What primitives will you need to get both asynchronous and synchronous message passing running? Three: send(message), receive(message) and request(message), as the send(message) primitive is/can be the same as the respond(message) primitive.

It is possible to get synchronous message passing via asynchronous message passing primitives and vice-versa. Message forwarders along with acknowledgment messages and messaging protocols can make this possible. Unfortunately, doing it this way is not efficient.

==The "Port" abstraction==
Many microkernels use the notion of port, which is a 'point for receiving messages', or a 'one-way message communication channel'. The idea of ports is that you no longer send a message to a thread but to a port (which is linked to a unique thread).

Ports can be allocated by servers at will and may restrict who can send and who cannot send on it. Using ports, the 'A listen to C, B sends to A' problem above can be solved by creating a new port p that A will open to C only (and sending a message to C to tell p's port number). A can now wait for C's message by receiving on p only. B's attempt to send a message on port q of A will not interfere.

To solve situations when a server is willing to receive messages from different ports (and synchronous messaging is used), we need a select-like interface that will allow reception of a message from any port p in a given set of ports.

==Mailboxes==
There may be some cases in which a thread will want to broadcast the same message to multiple threads at once, or perhaps a set of producer threads has to communicate with another set of consumer threads. In such cases, mailboxes can queue messages for multiple receivers. Mailboxes are similar to ports, except multiple recipients may retrieve messages instead of just one. A mailbox is not owned by a single thread, but is a common resource provided by the OS that's shared among the receiving threads. The order in which messages are received may either be ''first-in-first-out'' (FIFO) or based on message priority.

==Variable or Fixed Messages Size==
A message typically includes the ID of the emitting thread (or a replying port), a message code and a few arguments. Message queuing and dispatching code can be greatly simplified if message size is fixed.

When messages longer than the fixed-size are required, one will need to provide a way to describe the long message in the small structure. If copying a 4-word message forth and back doesn't imply excessive processing cost, it will be very different for the 1MB of data you send to a disk server. In that case, it is suggested to toy with paging in order to map the real data from the emitter to the receiver's address space.

== See Also ==

=== Articles ===
* [[Remote_Procedure_Call|Remote Procedure Call (RPC)]]
* [[Message Passing Tutorial]]
* [[IPC Data Copying methods]]

=== Threads ===
*[[Topic:10116|passing messages through registers]]
*[[Topic:10922|RPC message size, handling oversized messages]]

=== External Links ===
*[[Wikipedia:Message Passing | Message Passing]] on Wikipedia
*[[Wikipedia:Message queue | Message Queue]] on Wikipedia

[[Category:IPC]]
[[Category:OS theory]]
````

<a id="w001712"></a>

## W001712 — Real Mode

Bron-ID: W001712
Namespace: Main (0)
Revisie: 30882 | Datum: 2026-07-11T14:56:49Z
Bron: https://wiki.osdev.org/Real_Mode
Revisie-URL: https://wiki.osdev.org/index.php?title=Real_Mode&oldid=30882

````text
{{Historical}}

'''Real Mode''' is a simplistic 16-bit mode that is present on all x86 processors. Real Mode was the first x86 mode design and was used by many early operating systems before the birth of [[Protected Mode]]. For compatibility purposes, all x86 processors [[Boot Sequence|begin execution]] in Real Mode.

== Information ==
All modern operating systems (Windows, Linux, ...) run in [[Protected Mode]] due to the many limitations and problems that Real Mode
presents (see below, and the [[Real Mode OS Warning]] page). Older operating systems (such as DOS) and programs ran in Real Mode because it was the only mode available at the time. For information on how to switch from Real Mode to [[Protected Mode]], see the corresponding article.

Note: There is a mode called [[Virtual 8086 Mode]] which allows operating systems running in Protected mode to emulate the Real Mode segmented model for individual applications. This can be used to allow a [[Protected Mode]] operating system to still have access to e.g. BIOS functions, whenever needed.

Below you'll find a list of cons and pros. These are mostly 'in comparison to [[Protected Mode]]'.
 
=== Cons ===
* Less than 1 MB of RAM is available for use.
* There is no hardware-based memory protection (GDT), nor virtual memory.
* There are no built in security mechanisms to protect against buggy or malicious applications.
* The default CPU operand length is only 16 bits.
* The memory addressing modes provided are more restrictive than other CPU modes.
* Accessing more than 64k requires the use of segment registers that are difficult to work with.

=== Pros ===
* The [[BIOS]] installs device drivers to control devices and handle interrupts.
* [[BIOS#BIOS functions|BIOS functions]] provide operating systems with an advanced collection of low level API functions.
* Memory access is faster due to the lack of descriptor tables to check combined with smaller registers.

=== Common Misconception ===
Programmers often think that since Real Mode defaults to 16 bits, that the 32 bit registers are not
accessible. This is not true.

All of the 32-bit registers (EAX, ...) are still usable, by simply adding the "Operand Size Override
Prefix" (0x66) to the beginning of any instruction.  Your assembler is likely to do this for you, if
you simply try to use a 32-bit register.

== Memory Addressing ==
In Real Mode, there is a little over 1 MB of "addressable" memory (including the [[#High Memory Area|High Memory Area]]). See [[Detecting Memory (x86)]] and [[Memory Map (x86)]] to determine how much is actually '''usable'''. The usable amount will be much less than 1 MB. Memory access is done using [[Segmentation]] via a segment:offset system.

There are six 16-bit segment registers: CS, DS, ES, FS, GS, and SS. When using segment registers, addresses are given with the following notation (where 'Segment' is a value in a segment register and 'Offset' is a value in an address register):

<pre>
 12F3  :  4B27
   ^       ^
Segment   Offset
</pre>

Segments and Offsets are related to physical addresses by the equation:
  PhysicalAddress = Segment * 16 + Offset

Thus, 12F3:4B27 corresponds to the physical address 0x17A57. Any physical address can be represented in multiple ways, with different segments and offsets. For example, physical address 0x210 can be 0020:0010, 0000:0210, or 0021:0000.

=== The Stack ===
SS and SP are 16-bit segment:offset registers that specify a 20-bit physical address (described above), which is the current "top" of the stack. The stack stores 16-bit words, grows downwards, and must be aligned on a word (16-bit) boundary. It is used every time a program does a PUSH, POP, CALL, INT, or RET opcode and also when the BIOS handles any hardware interrupt.

=== High Memory Area ===
If you set DS (or any segment register) to a value of 0xFFFF, it points to an address that is 16 bytes below 1 MB. If you then use that segment register as a base, with an offset of 0x10 to 0xFFFF, you can access physical memory addresses from 0x100000 to 0x10FFEF. This (almost 64 kB) area above 1 MB is called the "High Memory Area" in Real Mode. Note that you have to have the [[A20]] address line activated for this to work.

=== Addressing Modes ===
Real Mode uses 16-bit addressing mode by default. Assembly programmers are typically familiar with the
more common 32-bit addressing modes, and may want to make adjustments -- because the registers that are available
in 16-bit addressing mode for use as "pointers" are much more limited. The typical programs that run
in Real Mode are often limited in the number of bytes available, and it takes one extra byte in each
opcode to use 32-bit addressing instead.

Note that you can still use 32-bit addressing modes in Real Mode, simply by adding the "Address Size
Override Prefix" (0x67) to the beginning of any instruction. Your assembler is likely to do this for you, if
you simply try to use a 32-bit addressing mode. But you are still constrained by the current "limit" for
the segment that you use in each memory access (always 64K in Real Mode -- [[Unreal Mode]] can be bigger).

* [BX + val]
* [SI + val]
* [DI + val]
* [BP + val]
* [BX + SI + val]
* [BX + DI + val]
* [BP + SI + val]
* [BP + DI + val]
* [address]

== Switching from [[Protected Mode]] to Real Mode ==
As noted above, it is possible for a Protected mode operating system to use [[Virtual 8086 Mode]] to access BIOS functions. However, VM86 mode has its own complications and difficulties. Some OS designers think that it is simpler and cleaner to temporarily return to Real Mode on those occasions when it is necessary to access a BIOS function. This requires creating a special Ring 0 program, and placing it in a physical memory address that can be accessed in Real Mode.

The OS usually needs to pass an information packet about which BIOS function to execute.

The program needs to go through the following steps:
# Disable the interrupts:
#* Turn off maskable interrupts using CLI.
#* Disable [[NMI]] (optional).
# Turn off [[Paging|paging]]:
#* Transfer control to a 1:1 page.
#* Ensure that the [[GDT]] and [[IDT]] are in a 1:1 page.
#* Clear the PG-flag in the zeroth control register.
#* Set the third control register to 0.
# Use GDT with 16-bit tables (skip this step if one is already available):
#* Create a new GDT with a 16-bit data and code segment:
#** Limit: 0xFFFFF
#** Base: 0x0
#** 16-bit
#** Privilege level: 0
#** Granularity: 0
#** Read and Write: 1
#* Load new GDT ensuring that the currently used selectors will remain the same (index in cs/ds/ss will be copy of original segment in new GDT)
# Far jump to 16-bit protected mode:
#* Far jump to 16-bit protected mode with a 16-bit segment index.
# Load data segment selectors with 16-bit indexes:
#* Load ds, es, fs, gs, ss with a 16-bit data segment.
# Load real mode IDT:
#* Limit: 0x3FF
#* Base 0x0
#* Use lidt
# Disable protected mode:
#* Set PE bit in CR0 to false.
# Far jump to real mode:
#* Far jump to real mode with real mode segment selector (usually 0).
# Reload data segment registers with real mode values:
#* Load ds, es, fs, gs, ss with appropriate real mode values (usually 0).
# Set stack pointer to appropriate value:
#* Set sp to stack value that will not interfere with real mode program.
# Enable interrupts:
#* Enable maskable interrupts with STI.
# Continue on in real mode with all BIOS interrupts.

=== x86 Assembly Example ===
<syntaxhighlight lang="nasm">
bits 16

idt_real:
	dw 0x3ff		; 256 entries, 4b each = 1K
	dd 0			; Real Mode IVT @ 0x0000

savcr0:
	dd 0			; Storage location for pmode CR0.

Entry16:
    ; We are already in 16-bit mode here!
	cli			        ; Disable interrupts.

	; Need 16-bit Protected Mode GDT entries!
	mov eax, DATASEL16	; 16-bit Protected Mode data selector.
	mov ds, eax
	mov es, eax
	mov fs, eax
	mov gs, eax
	mov ss, eax


	; Disable paging (we need everything to be 1:1 mapped).
	mov eax, cr0
	mov [savcr0], eax	; save pmode CR0
	and eax, 0x7FFFFFFe	; Disable paging bit & disable 16-bit pmode.
	mov cr0, eax

	jmp 0:GoRMode		; Perform Far jump to set CS.

GoRMode:
	mov sp, 0x8000		; pick a stack pointer.
	mov ax, 0		    ; Reset segment registers to 0.
	mov ds, ax
	mov es, ax
	mov fs, ax
	mov gs, ax
	mov ss, ax
	lidt [idt_real]     ; Load real mode IVT
	sti			        ; Restore interrupts -- be careful, unhandled interrupts will kill it.
</syntaxhighlight>

== See Also ==
=== Articles ===
* [[BIOS]]
* [[System Initialization (x86)]]
* [[Protected Mode]]

=== External Links ===
* [https://web.archive.org/web/20130609073242/http://www.osdever.net/tutorials/rm_addressing.php?the_id=50 The Workings of: x86-16/32 Real Mode Addressing] (2003)
* [http://therx.sourceforge.net/osdev/files/ia32_rm_addr.pdf The workings of IA32 real mode addressing and the A20 line] (2004)

=== References ===
* [http://www.intel.com/products/processor/manuals/ Intel® 64 and IA-32 Architectures Software Developer's Manual] (2011)
** Volume 3A: System Programming Guide, Part 1,Chapter 20:8086 EMULATION,which is a detailed reference about real mode using 32-bit addressing mode

[[Category:Real Mode]]
[[Category:X86]]
[[Category:X86 CPU]]
[[Category:Operating Modes]]
[[de:Real Mode]]
````

<a id="w001713"></a>

## W001713 — Protected Mode

Bron-ID: W001713
Namespace: Main (0)
Revisie: 30251 | Datum: 2026-03-16T15:51:19Z
Bron: https://wiki.osdev.org/Protected_Mode
Revisie-URL: https://wiki.osdev.org/index.php?title=Protected_Mode&oldid=30251

````text
'''Protected mode''' is the main operating mode of modern Intel processors (and clones) since the 80286 (16 bit). On 80386s and later, the 32 bit Protected Mode allows working with several virtual address spaces, each of which has a maximum of 4GB of addressable memory; and enables the system to enforce strict memory and hardware I/O protection as well as restricting the available instruction set via [[Security#Rings|Rings]].

A CPU that is initialized by the [[BIOS]] starts in [[Real Mode]]. Enabling Protected Mode unleashes the real power of your CPU. However, it will prevent you from using most of the BIOS interrupts, since these work in Real Mode (unless you have also written a [[Virtual 8086 Mode|V86]] monitor).

==Entering Protected Mode==

Before switching to protected mode, you must:
*Disable interrupts, including [[Non Maskable Interrupt|NMI]] (as suggested by Intel Developers Manual).
*Enable the [[A20 Line]].
*Load the [[Global Descriptor Table]] with segment descriptors suitable for code, data, and stack.

Whether the CPU is in Real Mode or in Protected Mode is defined by the lowest bit of the CR0 or MSW register. 

This example loads a descriptor table into the processor's GDTR register, and then sets the lowest bit of CR0:
<syntaxhighlight lang="nasm">
cli            ; disable interrupts
lgdt [gdtr]    ; load GDT register with start address of Global Descriptor Table
mov eax, cr0 
or al, 1       ; set PE (Protection Enable) bit in CR0 (Control Register 0)
mov cr0, eax

; Perform far jump to selector 08h (offset into GDT, pointing at a 32bit PM code segment descriptor) 
; to load CS with proper PM32 descriptor)
jmp 08h:PModeMain

PModeMain:
; load DS, ES, FS, GS, SS, ESP
</syntaxhighlight>

==See Also==
===Articles===
*[[Real Mode]]
*[[Virtual 8086 Mode]]

===External Links===
* [https://web.archive.org/web/20241222182652/http://www.osdever.net/tutorials/view/the-world-of-protected-mode The world of Protected mode] - tutorial on entering protected mode
* [https://web.archive.org/web/20130324180230/http://www.nondot.org/sabre/os/articles/ProtectedMode/ OSRC: protected mode]
* https://web.archive.org/web/20130512092408/http://home.swipnet.se/smaffy/asm/info/embedded_pmode.pdf - pragmatic tutorial on protected mode
* [http://www.brokenthorn.com/Resources/OSDev8.html Operating Systems Development - Protected Mode]
* [[Wikipedia:Protected_mode|Protected mode Wikipedia page]]
* [https://members.tripod.com/protected_mode/alexfru/pmtuts.html PMode tutorials in C & Asm]

[[Category:X86 CPU]]
[[Category:Operating Modes]]
````

<a id="w001715"></a>

## W001715 — Unreal Mode

Bron-ID: W001715
Namespace: Main (0)
Revisie: 30244 | Datum: 2026-03-16T15:45:50Z
Bron: https://wiki.osdev.org/Unreal_Mode
Revisie-URL: https://wiki.osdev.org/index.php?title=Unreal_Mode&oldid=30244

````text
'''Unreal mode''' consists of breaking the 64KiB limit of real mode segments (while retaining 16-bit instructions and the <tt>segment * 16 + offset</tt> addressing mode) by tweaking the [[Descriptor Cache|descriptor caches]].

==Usage==
Unreal mode is usually recommended in the two following cases:
* You're trying to extend a legacy 16-bit DOS program so that it can deal with larger data and neither [[Virtual 8086 Mode]], nor xms are suitable for your needs.
* You're trying to load something that will run in 32-bit mode which is larger than 640K (therefore you cannot load it in conventional memory) and you don't want to bother writing a protected mode disk driver yet, but you also want to avoid switching between real and protected mode to copy chunks from the conventional memory buffer into extended memory.

You still will not have full access to all physical RAM if you do not have the [[A20 Line]] enabled; all the "odd" 1 MiB blocks will be unavailable.

==Implementation==
To do this, you need to set the descriptor cache's limits for your segment register(s) to any value higher than 64KiB (usually a full 4GiB (<tt>0xffffffff</tt>)).

In protected mode, bits 3-15 in the segment registers represent an index into [[GDT|the global descriptor table]]. That's why in the following code 0x08 = 1000b gets you entry #1 (entry #0 is ALWAYS a null descriptor).

When (in protected mode) a segment register is loaded with a "selector", a "segment descriptor cache register" is filled with the descriptor's values, including the size (or limit). After the switch back to real mode, these values are not modified, regardless of what value is in the 16-bit segment register. So the 64KiB limit is no longer valid and 32-bit offsets can be used in Real Mode to actually access areas above 64KiB (<tt>segment * 16 + 32-bit offset</tt>).

===Big Unreal Mode===
This won't touch CS. <br />
Therefore IP is unaffected by all this, and the code itself is still limited to 64KiB.

<syntaxhighlight lang="nasm">
; Assembly example

; nasm boot.asm -o boot.bin
; partcopy boot.bin 0 200 -f0

org 0x7c00                ; add to offsets

start:
   xor ax, ax             ; make it zero
   mov ds, ax             ; DS=0
   mov ss, ax             ; stack starts at seg 0
   mov sp, 0x9c00         ; 2000h past code start, 
                          ; making the stack 7.5k in size

   cli                    ; no interrupts
   push ds                ; save real mode

   lgdt [gdtinfo]         ; load gdt register

   mov  eax, cr0          ; switch to pmode by
   or al,1                ; set pmode bit
   mov  cr0, eax
   jmp 0x8:pmode

pmode:
   mov  bx, 0x10          ; select descriptor 2
   mov  ds, bx            ; 10h = 10000b

   and al,0xFE            ; back to realmode
   mov  cr0, eax          ; by toggling bit again
   jmp 0x0:unreal

unreal:
   pop ds                 ; get back old segment
   sti

   mov bx, 0x0f01         ; attrib/char of smiley
   mov eax, 0x0b8000      ; note 32 bit offset
   mov word [ds:eax], bx

   jmp $                  ; loop forever

gdtinfo:
   dw gdt_end - gdt - 1   ;last byte in table
   dd gdt                 ;start of table

gdt:        dd 0,0        ; entry 0 is always unused
codedesc:   db 0xff, 0xff, 0, 0, 0, 10011010b, 00000000b, 0
flatdesc:   db 0xff, 0xff, 0, 0, 0, 10010010b, 11001111b, 0
gdt_end:

   times 510-($-$$) db 0  ; fill sector w/ 0's
   dw 0xAA55              ; Required by some BIOSes
</syntaxhighlight>

===Huge Unreal Mode===
Huge Unreal Mode enables code over 64KiB. However, it is more difficult to implement as real mode interrupts do not automatically save the high 16 bits of EIP. Initialization is simple though, you just load a code segment with a 4GiB limit:

<syntaxhighlight lang="nasm">
; Assembly example

; nasm boot.asm -o boot.bin
; partcopy boot.bin 0 200 -f0

org 0x7c00               ; add to offsets

start:   xor ax, ax       ; make it zero

   ...                    ; As before

   mov  cr0, eax
   jmp 0x8:pmode
pmode:
   mov  bx, 0x10          ; select descriptor 2, instead of 1
   mov  ds, bx            ; 10h = 10000b

   and al,0xFE            ; back to realmode
   mov  cr0, eax          ; by toggling bit again
   jmp 0x0:huge_unreal
huge_unreal:

   ...                    ;As before

gdtinfo:
   dw gdt_end - gdt - 1   ;last byte in table
   dd gdt                 ;start of table

gdt         dd 0,0        ; entry 0 is always unused
flatcode    db 0xff, 0xff, 0, 0, 0, 10011010b, 10001111b, 0
flatdata    db 0xff, 0xff, 0, 0, 0, 10010010b, 11001111b, 0
gdt_end:

   times 510-($-$$) db 0  ; fill sector w/ 0's
   dw 0xAA55              ; Required by some BIOSes
</syntaxhighlight>

WARNING: this may not work on some emulators or some hardware.

==Compiler Support==
===Smaller C===
The [[Smaller C]] compiler supports unreal mode. It produces [[MZ]] executables for unreal mode (can be loaded with [[BootProg]]).

The code and the stack are to be located below the 1MB mark and the stack size is limited by 64KB (IOW, there's nothing unusual about CS:(E)IP, SS:(E)SP, it's a natural setup for MZ executables in DOS). The DS and ES segment registers are set to 0, so C pointers can work as flat 32-bit physical addresses and address data or memory-mapped devices anywhere in the first 4GB of memory.

The startup code of these executables performs the necessary relocation (there are only custom relocations and no standard MZ relocations, which may simplify loading of the executables) and sets up unreal mode before passing control to the equivalent of ''main()''. See ''srclib/c0du.asm'' and other C/assembly code under ''srclib'' in the compiler source tree for how to write bits of assembly code for unreal mode (look for ''asm("inline asm code")'' under ''#ifdef __UNREAL__'').

You can try out unreal mode in DOS (e.g. in DOSBox, VirtualBox + FreeDOS) as the compiler fully supports the DOS + unreal mode combo in its C library. ''tests/vesalfb.c'' is a simple example of setting up a [[VESA]] graphics mode with the linear frame buffer enabled and drawing something on the screen in unreal mode.

For an example of an Unreal Mode [[bootloader]] implementation with Smaller C, look at [https://github.com/fysnet/FYSOS/tree/master/loader FYSOS].
[[Category:X86 CPU]]
[[Category:Operating Modes]]
[[Category:FAQ]]
````

<a id="w001716"></a>

## W001716 — File:VGA crtc.gif

Bron-ID: W001716
Namespace: File (6)
Revisie: 2538 | Datum: 2006-12-22T20:47:56Z
Bron: https://wiki.osdev.org/File:VGA_crtc.gif
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AVGA_crtc.gif&oldid=2538

````text
CRTC output diagram
````

<a id="w001717"></a>

## W001717 — CMOS

Bron-ID: W001717
Namespace: Main (0)
Revisie: 30263 | Datum: 2026-03-16T17:42:49Z
Bron: https://wiki.osdev.org/CMOS
Revisie-URL: https://wiki.osdev.org/index.php?title=CMOS&oldid=30263

````text
"CMOS" is a tiny bit of very low power static memory that lives on the same chip as the Real-Time Clock (RTC). 
It was introduced to IBM PC AT in 1984 which used Motorola MC146818A RTC.

CMOS (and the Real-Time Clock) can only be accessed through IO Ports 0x70 and 0x71. The function of the CMOS
memory is to store 50 (or 114) bytes of "Setup" information for the BIOS while the computer is turned off --
because there is a separate battery that keeps the Clock and the CMOS information active.

CMOS values are accessed a byte at a time, and each byte is individually addressable. Each CMOS address is
traditionally called a "register". The first 14 CMOS registers access and control the Real-Time Clock. In
fact, the only truly useful registers remaining in CMOS are the Real-Time Clock registers, and register 0x10.
All other registers in CMOS are almost entirely obsolete (or are not standardized), and are therefore useless.

== Non-Maskable Interrupts ==

For frugality in the olden days, many functions were merged together on chips where there was "room" -- even
if they did not belong together. An example is putting the [[A20]] address enable on the PS2 keyboard controller.
In the same way, the "NMI disable" control was put together with the CMOS controller and the Real-Time Clock.

NMI is meant to communicate a "panic" status from the hardware to the CPU in a way that the CPU cannot
ignore. It is typically used to signal memory errors. For more information about NMI, see the [[NMI]] article.

Whenever you send a byte to IO port 0x70, the high order bit tells the hardware whether to disable NMIs from
reaching the CPU. If the bit is on, NMI is disabled (until the next time you send a byte to Port 0x70).
The low order 7 bits of any byte sent to Port 0x70 are used to address CMOS registers.

== CMOS Registers ==

The most updated CMOS register map, showing all of the conflicting register definitions between the various
BIOSes, is in [[RBIL]] in the file called CMOS.LST.

=== Accessing CMOS Registers ===

Accessing CMOS is extremely simple, but you always need to take into account how you want to handle NMI.
You "select" a CMOS register (for reading or writing) by sending the register number to IO Port 0x70.
Since the 0x80 bit of Port 0x70 controls NMI, you always end up setting that, too. So your CMOS controller
always needs to know whether your OS wants NMI to be enabled or not. Selecting a CMOS register is done
as follows:

* <tt>outb (0x70, (NMI_disable_bit << 7) | (selected CMOS register number));</tt>

Once a register is selected, you either read the value of that register on Port 0x71 (with inb or an equivalent
function), or you write a new value to that register -- also on Port 0x71 (with outb, for example):

* <tt>val_8bit = inb (0x71);</tt>

Note1: Reading or writing Port 0x71 seems to default the "selected register" back to 0xD. So you need to
'''reselect''' the register every single time you want to access a CMOS register.

Note2: It is probably a good idea to have a reasonable delay after selecting a CMOS register on Port 0x70,
before reading/writing the value on Port 0x71.

==== Checksums ====

The proper functioning of the BIOS during bootup depends on the values in CMOS. So the values are protected
against random changes with checksums. It is very unwise ever to write a value into any of the CMOS registers
(except for the RTC) -- because when you change a value you also have to go fix a BIOS-specific checksum in a
different register -- or else the next boot will crash with an "invalid checksum" error. And since the checksum
is located at a proprietary BIOS-specific register number, good luck finding it.

=== Register 0x10 ===

This register contains the only CMOS value that an OS might ever find to be useful. It describes the "type"
of each of the two floppy drives that may be attached to the system. The high nibble describes the "master"
floppy drive on the primary bus, and the low nibble has an identical description for the "slave" floppy drive.

Value of each 4 bit nibble, and associated floppy drive type:
<pre>
Value   Drive Type
 00h	no drive
 01h	360 KB 5.25 Drive
 02h	1.2 MB 5.25 Drive
 03h	720 KB 3.5 Drive
 04h	1.44 MB 3.5 Drive
 05h	2.88 MB 3.5 drive
</pre>
Bits 0 to 3 = slave floppy type, bits 4 to 7 = master floppy type

=== Memory Size Registers ===

There are several CMOS registers that are standardized, and that seem to report useful information about the
total memory on the system. However, each of them is lacking vital information that your OS will need. It is
always better to use a BIOS function call to get information about memory than to use the information in CMOS.
See [[Detecting Memory (x86)]].

(Register 0x16 (high byte) | Register 0x15 (low byte)) << 10 = 640K = size of low memory (without taking the
[[Memory Map (x86)#EBDA|EBDA]] into account.

(Register 0x18 (high byte) | Register 0x17 (low byte)) << 10 = total memory between 1M and 16M, or maybe 65M
... usually. But this number is extra untrustworthy when the system has more than 64M, it ignores "memory
holes", it ignores memory mapped hardware, and it ignores memory reserved for important ACPI system tables.

(Register 0x31 (high byte) | Register 0x30 (low byte)) << 16 = total memory between 16M and 4G ... usually.
But this number is untrustworthy when the system has more than 4G, it ignores "memory holes", it ignores
memory mapped hardware, and it ignores memory reserved for important ACPI system tables.

=== Hard Disk Registers ===

There are many CMOS registers in various locations, used by various ancient BIOSes, to store a "hard disk type" or
other hard disk information. Any such information is strictly for use on obsolete CHS-based disk drives.
Better information can always be obtained via BIOS function INT13h AH=8, or by sending an ATA 'Identify'
command to the disk in [[ATA PIO Mode]].

== The Real-Time Clock ==

The RTC keeps track of the date and time, even when the computer's power is off. The only other way that a
computer used to be able to do this was to ask a human for the date/time on every bootup. Now, if the computer
has an internet connection, an OS has another (arguably better) way to get the same information.

The RTC also can generate clock ticks on IRQ8 (similarly to what the [[PIT]] does on IRQ0). The highest feasible
clock frequency is 8KHz. Using the RTC clock this way may actually generate more stable clock pulses than the PIT
can generate. It also frees up the PIT for timing events that really need near-microsecond accuracy.
Additionally, the RTC can generate an IRQ8 at a particular time of day. See the [[RTC]] article for more
detailed information about using RTC interrupts.

=== Getting Current Date and Time from RTC ===

To get each of the following date/time values from the RTC, you should first ensure that you won't be effected by an update ([[#RTC Update In Progress|see below]]). Then select the associated "CMOS register" in the
[[#Accessing CMOS Registers|usual way]], and read the value from Port 0x71.

<pre>
Register  Contents            Range
 0x00      Seconds             0–59
 0x02      Minutes             0–59
 0x04      Hours               0–23 in 24-hour mode, 
                               1–12 in 12-hour mode, highest bit set if pm
 0x06      Weekday             1–7, Sunday = 1
 0x07      Day of Month        1–31
 0x08      Month               1–12
 0x09      Year                0–99
 0x32      Century (maybe)     19–20?
 0x0A      Status Register A
 0x0B      Status Register B
</pre>

=== Century Register ===

Originally the RTC didn't have a century register at all. In the 1990s (as the year 2000 got closer) hardware manufacturers started realising that this might become a problem; so they starting adding century registers to their RTC. Unfortunately, because there was no official standard to follow, different manufacturers used different registers.

This meant that software didn't know if there was a century register, and (if there is) which register it might be. To fix that problem the ACPI specification included a "RTC century register" field at offset 108 in its "Fixed ACPI Description Table". If this field contains zero then the RTC doesn't have a century register, and if the field is non-zero then it contains the number of the RTC register to use for century.

If there is no century register then software can guess. For example, a piece of software written in 1990 can use the (2 digit) year register to determine the most likely century - if the RTC year register is higher than or equal to 90 then the year is probably be "19YY" and if the RTC year register is less than 90 than the year must be "20YY". In this way, software can correctly determine the century for up to 99 years after the software is written.

==== Century Register As Time and Date Sanity Check ====

If the CMOS/RTC has a century register, your software was released 2014, and the CMOS/RTC says the century and year are 2008; then obviously the CMOS/RTC must be wrong.

Similarly, people tend to update their OS occasionally. If the CMOS/RTC has a century register, your software was released 2014, and the CMOS/RTC says the century and year are 2154; then it's unlikely that the OS hasn't been updated for 140 years, and far more likely that the CMOS/RTC is wrong.

Essentially; the method (described above) for guessing the century when there is no century register is much more reliable than the CMOS/RTC century register (if it exists). This means that the century register (if/when present) can be used in reverse, as a way to check if the CMOS/RTC time and date are sane (or if the CMOS/RTC has a flat battery or something).

Basically, you'd guess the century (based on the software's release date and RTC's year), then check if the CMOS/RTC century is the same as your guess and if it's not then assume all CMOS/RTC time and date fields are invalid.

=== Weekday Register ===

The RTC chip is able to keep track of the current day of the week. All it does is increment its "Weekday" register at midnight and reset it to 1 if the incremented value would go above 7[https://groups.google.com/d/msg/alt.os.development/qPxWHKC48rw/yjAt-c8IAQAJ]. Unfortunately there is no guarantee that this register was ever set correctly by anything (including when the user changes the time and date using the BIOS configuration screen). It is entirely unreliable and should not be used.

The correct way to determine the current day of the week is to calculate it from the date (see [http://en.wikipedia.org/wiki/Weekday_determination the article on Wikipedia] for details of this calculation).

=== RTC Update In Progress ===

When the chip updates the time and date (once per second) it increases "seconds" and checks if it rolled over. If "seconds" did roll over it increases "minutes" and checks if that rolled over. This can continue through all the time and date registers (e.g. all the way up to "if year rolled over, increase century"). However, the RTC circuitry is typically relatively slow. This means that it's entirely possible to read the time and date while an update is in progress and get dodgy/inconsistent values (for example, at 9:00 o'clock you might read 8:59, or 8:60, or 8:00, or 9:00).

To help guard against this problem the RTC has an "Update in progress" flag (bit 7 of Status Register A). To read the time and date properly you have to wait until the "Update in progress" flag goes from "set" to "clear". This is not the same as checking that the "Update in progress" flag is clear.

For example, if code does "''while(update_in_progress_flag != clear)''" and then starts reading all the time and date registers, then the update could begin immediately after the "Update in progress" flag was checked and the code could still read dodgy/inconsistent values. To avoid this, code should wait until the flag becomes set and then wait until the flag becomes clear. That way there's almost 1 second of time to read all of the registers correctly.

Unfortunately, doing it correctly (waiting until the "Update in progress" flag becomes set and then waiting until it becomes clear) is very slow - it may take an entire second of waiting/polling before you can read the registers. There are 2 alternatives.

The first alternative is to rely on the "update interrupt". When the RTC finishes an update it generates an "update interrupt" (if it's enabled), and the IRQ handler can safely read the time and date registers without worrying about the update at all (and without checking the "Update in progress" flag); as long as the IRQ handler doesn't take almost a full second to do it. In this case you're not wasting up to 1 second of CPU time waiting/polling, but it may still take a full second before the time and date has been read. Despite this it can be a useful technique during OS boot - e.g. setup the "update interrupt" and its IRQ handler as early as you can and then do other things (e.g. loading files from disk), in the hope that the IRQ occurs before you need the time and date.

The second alternative is to be prepared for dodgy/inconsistent values and cope with them if they occur. To do this, make sure the "Update in progress" flag is clear (e.g. "''while(update_in_progress_flag != clear)''") then read all the time and date registers; then make sure the "Update in progress" flag is clear again (e.g. "''while(update_in_progress_flag != clear)''") and read all the time and date registers again. If the values that were read the first time are the same as the value that were read the second time then the values must be correct. If any of the values are different you need to do it again, and keep doing it again until the newest values are the same as the previous values.

=== Format of Bytes ===

There are 4 formats possible for the date/time RTC bytes:
* Binary or BCD Mode
* Hours in 12 hour format or 24 hour format

The format is controlled by Status Register B. On some CMOS/RTC chips, the format bits in Status Reg B
'''cannot be changed'''. So your code needs to be able to handle all four possibilities, and it should
not try to modify Status Register B's settings. So you always need to read Status Register B first, to
find out what format your date/time bytes will arrive in.

* Status Register B, Bit 1 (value = 2): Enables 24 hour format if set
* Status Register B, Bit 2 (value = 4): Enables Binary mode if set

Binary mode is exactly what you would expect the value to be. If the time is 1:59:48 AM, then the value
of hours would be 1, minutes would be 59 = 0x3b, and seconds would be 48 = 0x30.

In BCD mode, each of the two hex nibbles of the byte is modified to "display" a '''decimal''' number.
So 1:59:48 has hours = 1, minutes = 0x59 = 89, seconds = 0x48 = 72. To convert BCD back into a "good" binary value, use: binary = ((bcd / 16) * 10) + (bcd & 0xf) ''[Optimised: binary = ( (bcd & 0xF0) >> 1) + ( (bcd & 0xF0) >> 3) + (bcd & 0xf)]''.

24 hour time is exactly what you would expect. Hour 0 is midnight to 1am, hour 23 is 11pm.

12 hour time is annoying to convert back to 24 hour time. If the hour is pm, then the 0x80 bit is set on the hour byte. So you need to
mask that off. (This is true for '''both''' binary and BCD modes.) Then, midnight is 12, 1am is 1, etc.
Note that carefully: midnight is not 0 -- it is 12 -- this needs to be handled as a special
case in the calculation from 12 hour format to 24 hour format (by setting 12 back to 0)!

For the weekday format: Sunday = 1, Saturday = 7.

=== Interpreting RTC Values ===

On the surface, these values from the RTC seem extremely obvious. The main difficulty comes in deciding which timezone the values represent. The two possibilities are usually UTC, or the system's timezone, including Daylight Savings. See the [[Time And Date]] article for a much more complete discussion of how to handle this issue.

== Examples ==

=== Reading from the CMOS ===
<syntaxhighlight lang="c">
ReadFromCMOS (unsigned char array [])
{
   unsigned char tvalue, index;

   for(index = 0; index < 128; index++)
   {
      _asm
      {
         cli             /* Disable interrupts*/
         mov al, index   /* Move index address*/
         /* since the 0x80 bit of al is not set, NMI is active */
         out 0x70,al     /* Copy address to CMOS register*/
         /* some kind of real delay here is probably best */
         in al,0x71      /* Fetch 1 byte to al*/
         sti             /* Enable interrupts*/
         mov tvalue,al
       }

       array[index] = tvalue;
   }
}
</syntaxhighlight>

=== Writing to the CMOS ===
<syntaxhighlight lang="c">
WriteTOCMOS(unsigned char array[])
{
   unsigned char index;

   for(index = 0; index < 128; index++)
   {
      unsigned char tvalue = array[index];
      _asm
      {
         cli             /* Clear interrupts*/
         mov al,index    /* move index address*/
         out 0x70,al     /* copy address to CMOS register*/
         /* some kind of real delay here is probably best */
         mov al,tvalue   /* move value to al*/
         out 0x71,al     /* write 1 byte to CMOS*/
         sti             /* Enable interrupts*/
      }
   }
}
</syntaxhighlight>

=== Reading All RTC Time and Date Registers ===
<syntaxhighlight lang="c">
#define CURRENT_YEAR        2023                            // Change this each year!

int century_register = 0x00;                                // Set by ACPI table parsing code if possible

unsigned char second;
unsigned char minute;
unsigned char hour;
unsigned char day;
unsigned char month;
unsigned int year;

void out_byte(int port, int value);
int in_byte(int port);

enum {
      cmos_address = 0x70,
      cmos_data    = 0x71
};

int get_update_in_progress_flag() {
      out_byte(cmos_address, 0x0A);
      return (in_byte(cmos_data) & 0x80);
}

unsigned char get_RTC_register(int reg) {
      out_byte(cmos_address, reg);
      return in_byte(cmos_data);
}

void read_rtc() {
      unsigned char century;
      unsigned char last_second;
      unsigned char last_minute;
      unsigned char last_hour;
      unsigned char last_day;
      unsigned char last_month;
      unsigned char last_year;
      unsigned char last_century;
      unsigned char registerB;

      // Note: This uses the "read registers until you get the same values twice in a row" technique
      //       to avoid getting dodgy/inconsistent values due to RTC updates

      while (get_update_in_progress_flag());                // Make sure an update isn't in progress
      second = get_RTC_register(0x00);
      minute = get_RTC_register(0x02);
      hour = get_RTC_register(0x04);
      day = get_RTC_register(0x07);
      month = get_RTC_register(0x08);
      year = get_RTC_register(0x09);
      if(century_register != 0) {
            century = get_RTC_register(century_register);
      }

      do {
            last_second = second;
            last_minute = minute;
            last_hour = hour;
            last_day = day;
            last_month = month;
            last_year = year;
            last_century = century;

            while (get_update_in_progress_flag());           // Make sure an update isn't in progress
            second = get_RTC_register(0x00);
            minute = get_RTC_register(0x02);
            hour = get_RTC_register(0x04);
            day = get_RTC_register(0x07);
            month = get_RTC_register(0x08);
            year = get_RTC_register(0x09);
            if(century_register != 0) {
                  century = get_RTC_register(century_register);
            }
      } while( (last_second != second) || (last_minute != minute) || (last_hour != hour) ||
               (last_day != day) || (last_month != month) || (last_year != year) ||
               (last_century != century) );

      registerB = get_RTC_register(0x0B);

      // Convert BCD to binary values if necessary

      if (!(registerB & 0x04)) {
            second = (second & 0x0F) + ((second / 16) * 10);
            minute = (minute & 0x0F) + ((minute / 16) * 10);
            hour = ( (hour & 0x0F) + (((hour & 0x70) / 16) * 10) ) | (hour & 0x80);
            day = (day & 0x0F) + ((day / 16) * 10);
            month = (month & 0x0F) + ((month / 16) * 10);
            year = (year & 0x0F) + ((year / 16) * 10);
            if(century_register != 0) {
                  century = (century & 0x0F) + ((century / 16) * 10);
            }
      }

      // Convert 12 hour clock to 24 hour clock if necessary

      if (!(registerB & 0x02) && (hour & 0x80)) {
            hour = ((hour & 0x7F) + 12) % 24;
      }

      // Calculate the full (4-digit) year

      if(century_register != 0) {
            year += century * 100;
      } else {
            year += (CURRENT_YEAR / 100) * 100;
            if(year < CURRENT_YEAR) year += 100;
      }
}
</syntaxhighlight>

== See Also ==
* [[RTC]]

== External Links ==
* [https://web.archive.org/web/20111209041013/http://www-ivs.cs.uni-magdeburg.de/~zbrog/asm/cmos.html Old CMOS Map]
* [https://web.archive.org/web/20240119203005/http://www.bioscentral.com/misc/cmosmap.htm Better CMOS Map]
* [http://bitsavers.trailing-edge.com/pdf/ibm/pc/at/1502494_PC_AT_Technical_Reference_Mar84.pdf IBM PC AT Technical Reference]
* [http://web.stanford.edu/class/cs140/projects/pintos/specs/mc146818a.pdf MC146818A REAL-TIME CLOCK PLUS RAM (RTC)]
* [http://www.bitsavers.org/pdf/ibm/pc/ps2/PS2_Model_50_Technical_Reference_May88.pdf IBM PS/2 Technical Reference]
* [https://www.singlix.com/trdos/archive/pdf_archive/real-time-clock-nmi-enable-paper.pdf Intel Application Note: Accessing the Real Time Clock Registers and NMI Enable Bit]

[[Category:X86]]
[[Category:Time]]
[[de::CMOS]]
````

<a id="w001718"></a>

## W001718 — File:Stack.png

Bron-ID: W001718
Namespace: File (6)
Revisie: 2548 | Datum: 2006-12-23T07:37:51Z
Bron: https://wiki.osdev.org/File:Stack.png
Revisie-URL: https://wiki.osdev.org/index.php?title=File%3AStack.png&oldid=2548

````text
Adaptation of [[:Image:Data_stack.png]]
````

<a id="w001719"></a>

## W001719 — Interrupt Service Routines

Bron-ID: W001719
Namespace: Main (0)
Revisie: 30979 | Datum: 2026-09-27T19:56:54Z
Bron: https://wiki.osdev.org/Interrupt_Service_Routines
Revisie-URL: https://wiki.osdev.org/index.php?title=Interrupt_Service_Routines&oldid=30979

````text
The [[:Category:x86|x86]] architecture is an [[interrupt]] driven system. External events trigger an interrupt — the normal control flow is interrupted and an '''Interrupt Service Routine''' (ISR) is called.

Such events can be triggered by hardware or software. An example of a hardware interrupt is the keyboard: every time you press a key, the keyboard triggers IRQ1 (Interrupt Request 1), and the corresponding interrupt handler is called. Timers and disk request completion are other possible sources of hardware interrupts.

Software driven interrupts are triggered by the <tt>int</tt> opcode; e.g. the services provided by MS-DOS are called by the software triggering <tt>INT 21h</tt> and passing the applicable parameters in CPU registers.

For the system to know which interrupt service routine to call when a certain interrupt occurs, offsets to the ISRs are stored in the [[IDT|Interrupt Descriptor Table]] when you're in [[Protected mode]], or in the [[Interrupt Vector Table]] when you're in [[Real Mode]].

An ISR is called directly by the CPU, and the protocol for calling an ISR differs from calling e.g. a C function. Most importantly, an ISR has to end with the <tt>iret</tt> opcode (or <tt>iretq</tt> in long mode—yes, even when using intel syntax), whereas usual C functions end with <tt>ret</tt> or <tt>retf</tt>. The obvious but nevertheless wrong solution leads to one of the most "popular" triple-fault errors among OS programmers.

== When the Handlers are Called ==
=== x86 ===
When the CPU calls the interrupt handlers, the CPU pushes these values onto the stack in this order:

<pre>
EFLAGS -> CS -> EIP
</pre>

The CS value is padded with two bytes to form a doubleword.

If the gate type is not a trap gate, the CPU will clear the interrupt flag. If the interrupt is an exception, the CPU will push an error code onto the stack, as a doubleword.

The CPU will load the segment-selector value from the associated IDT descriptor into CS.

=== x86-64 ===
When the CPU calls the interrupt handlers, it changes the value in the RSP register to the value specified in the IST, and if there is none, the stack stays the same. Onto the new stack, the CPU pushes these values in this order:

<pre>
SS:RSP (original RSP) -> RFLAGS -> CS -> RIP
</pre>

CS is padded to form a quadword.

If the interrupt is called from a different ring, SS is set to 0, indicating a null selector. The CPU will modify the RFLAGS register, setting the TF, NT, and RF bits to 0. If the gate type is an interrupt gate, the CPU will clear the interrupt flag.

If the interrupt is an exception, the CPU will push an error code onto the stack, padded with bytes to form a quadword.

The CPU will load the segment-selector value from the associated IDT descriptor into CS, and check to ensure that CS is a valid code segment selector.

== The Problem ==
Many people shy away from assembly, and want to do as much as possible in their favorite high-level language. [[GCC]] (as well as other compilers) allow you to add inline assembly, so many programmers are tempted to write an ISR like this:

<syntaxhighlight lang="C">
/* How NOT to write an interrupt handler */
void interrupt_handler(void)
{
    asm("pushad"); /* Save registers. */
    /* do something */
    asm("popad");  /* Restore registers. */
    asm("iret");   /* This will triple-fault! */
}
</syntaxhighlight>

This cannot work. The compiler doesn't understand what is going on. It doesn't understand that the registers and stack are required to be preserved between the asm statements; the optimizer will likely corrupt the function. Additionally, the compiler adds stack handling code before and after your function, which together with the iret results in assembly code resembling this:

<syntaxhighlight lang="asm">
push   %ebp
mov    %esp,%ebp
sub    $<size of local variables>,%esp
pushad
# C code comes here
popad
iret
# 'leave' if you use local variables, 'pop %ebp' otherwise.
leave
ret
</syntaxhighlight>

It should be obvious how this messes up the stack (ebp gets pushed but never popped). Don't do this. Instead, these are your options.

== Solutions ==
=== Plain Assembly ===

Learn enough about assembly to write your interrupt handlers in it. ;-)

=== Two-Stage Assembly Wrapping ===

Write an assembly wrapper calling the C function to do the real work, and then doing the iret.

<syntaxhighlight lang="asm">
; filename: isr_wrapper.s
.globl   isr_wrapper
.align   4

isr_wrapper:
    pushad
    cld    ; C code following the sysV ABI requires DF to be clear on function entry
    call interrupt_handler
    popad
    iret
</syntaxhighlight>
<syntaxhighlight lang="C">
/* filename: interrupt_handler.c */
void interrupt_handler(void)
{
    /* do something */
}
</syntaxhighlight>

=== Compiler Specific Interrupt Directives ===

Some compilers for some processors have directives allowing you to declare a routine interrupt, offer a #pragma interrupt, or a dedicated macro. Clang 3.9, Borland C, Watcom C/C++, Microsoft C 6.0 and Free Pascal Compiler 1.9.* upward and GCC offer this. Visual C++ offers an alternative shown under '''Naked Functions''':

==== Rust ====

Using a nightly compiler and the [https://doc.rust-lang.org/unstable-book/language-features/abi-x86-interrupt.html `abi_x86_interrupt` feature](note that at the moment, April 28th 2025, there is no information on this page except a link to the tracking issue), Rust supports this. Note this is still experimental and has some known issues(read [https://github.com/rust-lang/rust/issues/40180 the tracking issue thread].

<syntaxhighlight lang="Rust">
#![feature(abi_x86_interrupt)]

#[repr(C)]
struct InterruptStackFrame {
    ip: usize,
    cs: usize,
    flags: usize,
    sp: usize,
    ss: usize,
}

pub unsafe extern "x86-interrupt" fn interrupt_handler(stack_frame: InterruptStackFrame) {
    // do something
}
</syntaxhighlight>

==== Clang ====

As of version 3.9 it supports [https://clang.llvm.org/docs/AttributeReference.html#interrupt-x86 interrupt attribute] for x86/x86-64 targets.

<syntaxhighlight lang="C">
struct interrupt_frame
{
    uword_t ip;
    uword_t cs;
    uword_t flags;
    uword_t sp;
    uword_t ss;
};

__attribute__ ((interrupt))
void interrupt_handler(struct interrupt_frame *frame)
{
    /* do something */
}
</syntaxhighlight>

==== Borland C ====

<syntaxhighlight lang="C">
/* Borland C */
void interrupt interrupt_handler(void)
{
    /* do something */
}
</syntaxhighlight>

==== Watcom C/C++ ====

<syntaxhighlight lang="C">
/* Watcom C/C++ */
void _interrupt interrupt_handler(void)
{
    /* do something */
}
</syntaxhighlight>

==== Zig (LLVM only) ====

<syntaxhighlight lang="Zig">
export fn interruptHandler(frame: *const InterruptFrame) callconv(.{ .x86_64_interrupt = .{} }) void {
    _ = frame; // do something
}
</syntaxhighlight>

==== Naked Functions ====

Some compilers can be used to make interrupt routines, but require you to manually handle the stack and return operations. Doing so requires that the function is generated without an epilogue or prologue. This is called making the function ''naked'' — this is done in Visual C++ by adding the attribute ''_declspec(naked)'' to the function. You need to verify that you do include a return operation (such as ''iretd'') as that is part of the epilogue that the compiler has now been instructed to not include.

If you intend to use local variables, you must set up the stack frame in the manner which the compiler expects; as ISRs are non-reentrant, however, you can simply use static variables.

==== Visual C++ ====

Visual C++ also supplies the __LOCAL_SIZE assembler macro, which notifies you how much space is required by the objects on the stack for the function.

<syntaxhighlight lang="C">
/* Microsoft Visual C++ */
void _declspec(naked) interrupt_handler()
{
    _asm pushad;

    /* do something */

    _asm{
        popad
        iretd
    }
}
</syntaxhighlight>

==== GCC / G++ ====

[https://gcc.gnu.org/onlinedocs/gcc/x86-Function-Attributes.html#x86-Function-Attributes The online docs] say that by using GCC's function attributes, they have added the ability to write interrupt handlers in C interface by using __attribute__((interrupt)). So instead of:

<syntaxhighlight lang="C">
/* BLACK MAGIC – strongly discouraged! */
void interrupt_handler() {
    __asm__("pushad");
    /* do something */
    __asm__("popad; leave; iret"); /* BLACK MAGIC! */
}
</syntaxhighlight>

You can have the correct (GCC) form:

<syntaxhighlight lang="C">
struct interrupt_frame;

__attribute__((interrupt)) void interrupt_handler(struct interrupt_frame* frame)
{
    /* do something */
}
</syntaxhighlight>

The documentation for GCC states that if the interrupt attribute is used, the iret instruction will be used instead of ret on x86 and x86-64 architectures. It also says, "Since GCC doesn’t preserve SSE, MMX nor x87 states, the GCC option -mgeneral-regs-only should be used to compile interrupt and exception handlers."

===== Black Magic =====

Look at the faulty code [[Interrupt_Service_Routines#The_Problem|above]], where the proper C function exit code was skipped, screwing up the stack. Now, consider this code snippet, where the exit code is added manually:

<syntaxhighlight lang="C">
/* BLACK MAGIC – strongly discouraged! */
void interrupt_handler() {
    __asm__("pushad");
    /* do something */
    __asm__("popad; leave; iret"); /* BLACK MAGIC! */
}
</syntaxhighlight>

The corresponding output would look somewhat like this:

<syntaxhighlight lang="asm">
push   %ebp
mov    %esp,%ebp
sub    $<size of local variables>,%esp
pushad
# C code comes here
popad
leave
iret
leave # dead code
ret   # dead code
</syntaxhighlight>

This assumes that <tt>leave</tt> is the correct end-of-function handling — you are doing the function return code "by hand", and leave the compiler-generated handling as "dead code". Needless to say, such assumptions on compiler internals are dangerous. This code can break on a different compiler, or even a different version of the same compiler. It is therefore strongly discouraged, and listed only for completeness.

===== Assembly Goto =====
:''Full article: [[ISRs_PIC_And_Multitasking|ISRs, PIC, And Multitasking]]''

Since version 4.5, GCC supports the "asm goto" statement. It can be used to make ISRs as functions which return the correct address of the ISR entry point.

[[Category:Interrupts]]
[[Category:x86]]
````

<a id="w001723"></a>

## W001723 — Talk:Message Passing

Bron-ID: W001723
Namespace: Talk (1)
Revisie: 2579 | Datum: 2006-12-27T05:26:58Z
Bron: https://wiki.osdev.org/Talk:Message_Passing
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AMessage_Passing&oldid=2579

````text
== Rv edit by Brynet-Inc, 'queuing' and 'oversized' are the correct ==
They are correct!!

Unless you mean we have to follow US English.. Isn't British/Canadian English more correct?
--''[[User:Brynet-Inc|Brynet-Inc]]''

:I know i am slightly biased... being English, but i prefer the Real english. It always seems more proffesional to use the original. I find using american english akin to typing with a dialect. -[[User:Tyler|Tyler]] 23:00, 26 December 2006 (CST)

:"queueing" I am ''personally'' alright with (I am Canadian as well); however, I must point out that it is not alright to convert between two dialects unless the wiki has guidelines for which should be used. As two thirds of native speakers of English live in the United States wikis generally opt for American English. Oversized on the other hand is one word in both (I have seen it hyphenated but I do not believe it to be correct). Furthermore it is the title of a forum topic and I would argue against changing it even if it was undeniably misspelled (i.e. oversizd). -[[User:Jhawthorn|Jhawthorn]] 23:26, 26 December 2006 (CST)
````

<a id="w001725"></a>

## W001725 — Universal Serial Bus

Bron-ID: W001725
Namespace: Main (0)
Revisie: 29542 | Datum: 2025-06-08T17:41:27Z
Bron: https://wiki.osdev.org/Universal_Serial_Bus
Revisie-URL: https://wiki.osdev.org/index.php?title=Universal_Serial_Bus&oldid=29542

````text
The Universal Serial Bus was first introduced in 1994 with the intention of replacing various specialized interfaces, and to simplify the configuration of communication devices.  The communication industry did not develop as the USB-IF foresaw, but the various transfer modes that USB introduced allowed it to become one of the most popular standards in use today.  Virtually every modern computer supports USB.

== Introduction ==
Despite how attractive USB support is, the 650-page USB 2.0 specification manages to deter even some of the most driven hobbyists (especially if English isn't their primary language).  Not only is the USB 2.0 specification long, but it's a prerequisite for the [[XHCI]], [[EHCI]], [[UHCI]], and [[OHCI]] specifications, which define the actual hardware OSes interface with. Furthermore, the USB specification defines a plethora of terms, some used interchangeably and seemingly lazily; as a lengthy technical document, it is neither easy nor practical to flip back and forth to clarify a confusing term or concept.

=== What this text covers ===
The truth is that a software developer doesn't need to read the entire USB 2.0 specification; there are sections specific to hardware developers, for example.  The information presented here attempts to summarize chapters 4, 5, and 8 through 10.

Chapter 11 is specific to hubs and is also essential for a full USB 2.0 implementation, however it is almost as long as chapters 4, 5, 8, 9, and 10 combined, and could be regarded as the documentation for a specific (albeit special) class of USB devices.  Chapter 11 is covered thusly in its own wiki entry, [[USB Hubs]].  Even so, some concepts which pertain to USB hubs are briefly discussed where relevant in this article.

Ideally, the text here will establish familiarity with the terms and concepts that a hobby OS developer needs to begin implementing USB support and, if necessary, easily parse the USB specification without becoming intimidated by the amount of information.  At the very least, the system programmer should keep a copy of the USB 2.0 specification for reference while working with USB-related hardware.

Fortunately, all of the necessary documentation is available for free (see [http://wiki.osdev.org/Universal_Serial_Bus#External_Links Links]).

=== What this text does not cover ===
Please note that USB, unlike other standards like [[VGA]] or [[PCI]], is agnostic of the hardware interface to the system bus (and, by extension, to the operating system).  Such an interface is provided by one or more [[#Host_Controllers|USB host controllers]] and is defined by the appropriate documentation.  Therefore, one should not expect this text to discuss specifics or code samples (e.g., as one finds in the wiki entries about [[VGA]] or [[PCI]]) detailing how the operating system initiates and maintains communication with USB devices.  Although such information may be found on wiki entries discussing a particular [[#Host_Controller_Driver|Host Controller Driver]], those wiki entries assume an understanding of the concepts and terms discussed here.

== Host Controllers ==
The '''Host Controller''' is the USB interface to the host computer system.  In other words, the host controller is what the system software uses to communicate with USB devices.

=== USB 1.0 Host Controllers ===
{{Main|Universal Host Controller Interface}}
{{Main|Open Host Controller Interface}}

Intel brought USB 1.0 to the market with its '''Universal Host Controller Interface''' ('''UHCI'''), while Compaq, Microsoft, and National Semiconductors did the same with their '''Open Host Controller Interface''' ('''OHCI''').  Naturally, the two interfaces are incompatible, and to make things worse, VIA Technologies licensed Intel's UHCI standard, thereby ensuring that both standards survived.  Typically, an on-board chip set will contain a UHCI implementation, whereas a peripheral card typically implements the OHCI standard (but this is by no means a guarantee).

=== USB 2.0 Host Controllers ===
{{Main|Enhanced Host Controller Interface}}

[[Image:PortRoutingBlockDiagram.gif|frame|Figure 1: Block Diagram of Port Routing Behavior]]
In designing USB 2.0, the USB-IF insisted on a single implementation.  That single implementation is Intel's '''Enhanced Host Controller Interface''' ('''EHCI''').  However, even though the USB 2.0 specification requires that a USB 2.0 interface support USB 1.0 devices, this doesn't mean that the EHCI must support USB 1.0 devices, and in fact, it doesn't.  Each EHCI host controller is accompanied by (usually several) UHCI and/or OHCI host controllers.  When a USB 1.0 device is attached, the EHCI simply hands control over to a '''companion controller'''.  Refer to figure 1 for a simple block diagram implementation of this behavior.  Therefore, the system programmer must support all three standards in order to support USB 2.0. 

The EHCI host controller only handles USB 1.0 devices if they are attached indirectly through a USB 2.0 hub.  The specifics of handling USB 1.0 devices attached to a USB 2.0 hub are briefly discussed and illustrated in the [[#Hubs|hubs]] section, and in more detail in the wiki entry for [[USB Hubs]]. Note that some newer chipsets like the Intel 5-series chipsets do not have companion controllers at all and instead have internal "rate matching" hubs that all USB devices go through.

=== USB 3.0 Host Controllers ===
{{Main|eXtensible Host Controller Interface}}

Like its predecessor USB 2.0, USB 3.0 has only one host controller specification: Intel's '''eXtensible Host Controller Interface'''. Unlike its predecessor EHCI, however, xHCI controllers can and do interface with USB 1.0 and 2.0 devices without the use of companion controllers. Even on early hardware where there was both an EHCI and xHCI controller included (so that OSes which did not yet support xHCI could still use at least some USB devices), ports attached to the EHCI controller could generally be "re-routed" to the xHCI controller, and the EHCI controller disabled entirely.

Also unlike its predecessors, xHCI was designed with some degree of ''forwards compatibility'', so that revisions to the USB specification can be made without designing a new host controller interface (for instance, USB 3.1 and 3.2 add new speeds, with only minor updates to the specification to match them.) Unfortunately, this means that xHCI bears only a passing resemblance to the controllers that came before it, and make it challenging to write drivers for.

== Basic Concepts and Nomenclature ==
The USB is a polled bus, meaning the host controller must initiate all transfers.  Do not mistake this to mean that the system software must poll the USB.  The host controller takes care of polling the bus and can be programmed to issue interrupts to the OS whenever the bus needs attention.

=== USB System ===
[[Image:USBSystem.gif|frame|Figure 2: USB System Illustration]]
A '''USB System''' consists of three discrete parts: the '''USB device(s)''', the '''USB interconnect''', and the '''USB host'''.  Figure 2 illustrates a USB System.

==== USB Device(s) ====
'''USB devices''' are classified as either a '''hub''' or a '''function''' (not to be confused with a program procedure).  Hubs provide additional attachment points, whereas functions provide capabilities to the system.  Some devices may implement several functions and an embedded hub in one physical package.  These are called '''compound devices'''.

===== Functions =====
All functions understand the USB protocol, respond to standard operations (e.g, configuration or reset), and describe capabilities to the USB host.

There are four speed classes of functions:
* '''Super-speed''' functions operate at up to 5 Gb/s.
* '''High-speed''' functions operate at up to 480 Mb/s.
* '''Full-speed''' functions operate at up to 12 Mb/s.
* '''Low-speed''' functions operate at up to 1.5 Mb/s.

The original USB specification defined low- and full-speed devices, while USB 2.0 added high-speed devices and USB 3.0 added super-speed devices.

===== Hubs =====
In a high-speed system, a high-speed hub plays a special role.  Since the high-speed hub establishes a high-speed transfer rate with the host, it must isolate any full- or low-speed signaling from both the host and any attached high-speed devices.

To better understand, consider that the EHCI controller is accompanied by one or more companion controllers, as illustrated in figure 1 above.  When a full- or low-speed device is attached directly to the root hub, the EHCI controller can relinquish ownership of that specific port to a companion controller as seen in figure 3.  However, if a high-speed hub is connected to a port, as in Figure 4, then the EHCI controller must retain ownership of the port because it is a high-speed device.  Now suppose other high-speed devices are attached to the high-speed hub in figure 4; obviously the EHCI controller retains control as in figure 5.

But what happens when a full- or low-speed device is connected to the high-speed hub in figure 5?  If the EHCI controller were to relinquish ownership of the port, the high-speed devices will no longer be able to operate at high-speed, if at all, as in figure 6.  Instead, the host controller and the hub support a special type of transaction called a split transaction.  A '''split transaction''' involves only the host controller and a high-speed hub; it is transparent to any devices.  This scheme of using split-transaction to support low- and full-speed devices on a high-speed hub is illustrated in figure 7.

Note that some newer chipsets like the Intel 5-series chipsets do not have companion controllers at all and instead have internal "rate matching" hubs that all USB devices go through.

<gallery perrow=5>
Image:LFSpeedDevToHSPort.gif|Figure 3: Low- or Full-speed device connected to a high-speed capable USB port
Image:HSHubToHSPort.gif|Figure 4: High-speed hub connected to a high-speed capable USB port
Image:HSDevsToHSHubToHSPort.gif|Figure 5: High-speed devices connected to a high-speed hub which is connected to a high-speed USB port
Image:LFHSDevsHSHubHSPortIncorrect.gif|Figure 6: Incorrect illustration of Low- and Full-speed devices on a high-speed bus
Image:LFHSDevsHSHubHSPortCorrect.gif|Figure 7: Correct illustration of split transactions allowing Low- and Full-speed devices on a high-speed bus
</gallery>

[[Image:USBTopology.gif|thumb|right|Figure 8: USB Topology]]

==== USB Interconnect ====
The '''USB interconnect''' provides a connection from the USB device(s) to the USB host.  Physically, the USB interconnect is a tiered star topology.  A maximum of seven tiers are allowed, and the root hub occupies the first tier.  Since compound devices contain an embedded hub, a compound device cannot be attached in tier 7.  Figure 8 illustrates a USB topology (taken from Figure 4-1 of the USB 2.0 specifications).

==== USB Host ====
A USB system contains only one '''USB host'''.  The host interfaces with the USB interconnect via a host controller.  The host includes an embedded hub called the '''root hub''' which provides one or more '''attachment points''', or '''ports'''.

=== USB Communication Flow ===
[[Image:USBDataFlow.gif|frame|right|Figure 9: Illustration of USB Communication Flow]]
Figure 9 illustrates the concepts of USB communication flow and is taken from Figure 5-10 of the USB 2.0 Specifications.

==== Device Endpoints and Endpoint Numbers ====
Each USB device contains a collection of endpoints.  Every endpoint has the following characteristics:
* Bus access frequency/latency requirement
* Bandwidth requirement
* A unique device-determined identifier called the endpoint number
* Error handling behavior requirements
* Maximum packet size the endpoint can send or receive
* The transfer type of the endpoint
* Device-determined direction of data transfer:
** '''Input''': from the device to the host
** '''Output''': from the host to the device

As an example, consider an “all-in-one” printer/scanner device.  Such a device may implement an endpoint number for printing functionality, and a separate endpoint number for scanning functionality.

Although endpoints have a specific direction, two endpoints may have the same endpoint number but opposing data transfer directions.  All functions implement two such endpoints with the endpoint number 0.  Only endpoints with the endpoint number 0 may be accessed as soon as the device is powered and has received a bus reset; all other endpoints are in an undefined state until the device is configured.

Besides the two required endpoints, functions may implement additional endpoints as necessary, with the following limitations:
* Low-speed functions may implement up to two additional endpoints.
* Full- and high-speed devices may implement up to 15 additional input endpoints and 15 additional output endpoints.  This is a physical limitation of the USB protocol and is discussed under [[#Endpoint_field|Endpoint_Field]].

==== Endpoint Zero ====
All USB devices implement input and output endpoints with an endpoint number of 0.  These endpoints are collectively known as the '''default control pipe'''.  Endpoints with an endpoint number 0 are special in that they are accessible whenever the device is attached, powered and has received a bus reset.

In the interest of backwards compatibility, all high-speed functions must support these endpoints even when connected to a hub operating at full-speed.  This means that high-speed devices must be able to reset at full-speed, as well as respond successfully to standard requests at full-speed.  The high-speed device is not, however, required to support its intended functionality at full-speed.  This allows USB 1.0 systems to identify a USB 2.0 device and alert the user if the device cannot function properly at full-speed

==== Pipes ====
A pipe associates software on the host (specifically, a buffer on the host) with an endpoint on a device.  

There are two kinds of pipe communication modes:
* '''Stream pipes''' impose no structure on the data being transferred.  Stream pipes are always '''uni-directional''' in their communication flow.
* '''Message pipes''' impose some structure on the data being transfered.  Message pipes are '''bi-directional''', however data may predominantly transfer in one direction.

Pipes also have the following attributes:
* A claim on bus access and bandwidth usage
* A transfer type
* The associated endpoint's characteristics

Data flow in one pipe is independent of data flow in any other pipe.  Most pipes are available after a device has been configured, however the '''default control pipe''' always exists after a USB device is powered and has received a bus reset.

==== Default Control Pipe ====
The '''default control pipe''' is a special type of message pipe that is always accessible once a device is powered and has received a bus reset.  Thus, the default control pipe provides a means to identify and configure devices so that additional endpoints, if any, are made available.

The information required to completely identify a device is associated with the default control pipe; such information falls into the following categories:
* '''Standard''' information is common among all USB devices.
* '''Class''' information depends on the class of the USB device, as identified by the standard information.
* '''USB Vendor''' information is free for use by the hardware vendor.

=== Basics of USB Transfers ===
Most USB transactions consist of three packets:
* A '''token packet''' indicates the type and direction of the transaction, the device address, and an endpoint number.
* Depending on the direction of the transaction, either the host or the function sends a '''data packet''' (which may simply indicate that there is no data to send).
* The receiving device responds with a '''handshake packet''' to indicate if the transfer was successful.

USB supports four basic types of data transfer which take place via pipes.  A single pipe supports only (and exactly) one transfer type for any given device configuration.  That is, a function may provide a means to change the transfer type of a device-implemented endpoint number.

Briefly, the four basic transfer types are:
* '''Control Transfers''' provide lossless transmissions and are used to configure a device.  Thus, all USB devices must support control transfers at least via the default control pipe.
* '''Bulk Data Transfers''' provide lossless, sequential transmissions and are typically used to transfer large amounts of data.
* '''Interrupt Data Transfers''' provide reliable, limited-latency transmissions typically needed by human input devices such as a mouse or a joystick.
* '''Isochronous Data Transfers''', also called '''Streaming Real-time Transfers''', negotiate a required bandwidth and latency when initializing the transfer.  This transfer type is predominantly used for such applications as streaming audio.  Since data-delivery rate is considered more important than data integrity for this type of transfer, it does not provide any type of error checking or correction mechanism.

==== Control Transfers ====
Control transfers support configuration/command/status type communication flow.  The host initiates a control transfer with a SETUP bus transaction to the function, which establishes details of the intended data transfer such as whether the host wishes to send or receive data.  Next, zero or more DATA transactions take place in the appropriate direction.  Finally, a STATUS transaction from the function to the host indicates whether the transfer was successful.

Clearly, control transfers adhere to a USB-defined structure, so it should come as no surprise that control transfers may only be carried out via messages pipes.

Neither a function nor the host are guaranteed any specific latency or bandwidth for control transfers.

===== Maximum Data Payload Size =====
An endpoint used for a control transfer specifies the '''maximum data payload size''' that it can accept or transmit to the bus.  The allowable maximum data payload sizes depend on the speed of the device:
* High-speed device endpoints may only select a maximum data payload size of 64 bytes.
* Full-speed device endpoints may select a maximum data payload size of 8, 16, 32, or 64 bytes.
* Low-speed device endpoints may only select a maximum data payload size of 8 bytes.

A control transfer always uses it's maximum data payload size for data payloads unless the data payload is less than the maximum data payload size.  That is, if an endpoint has a maximum data payload size of 64 bytes, and a control transfer intends to transmit 100 bytes, the first data payload must contain 64 bytes and no less.  The remaining 36 bytes are transferred in the second payload and need not be padded to 64 bytes.  When the host receives a data payload less than the maximum data payload, the host may consider the transfer complete.

A SETUP transaction's data payload is always 8 bytes and thus receivable by the endpoint of any USB device.  Consequently, the host may query the appropriate descriptor from a newly-attached full-speed device during configuration in order to determine the maximum data payload size for any endpoint; the host can then adhere to that maximum for any future transmissions.

===== Transmission Errors =====
====== Too Much Data ======
When transferring from host to device, if the host sends more data than negotiated during the SETUP transaction (i.e., the device receives more data than it expects; specifically, the host does not advance to the STATUS stage when the device expects), the device endpoint halts the pipe.

When transferring from device to host, if the device sends more data than negotiated during the SETUP transaction (i.e., the host receives an extra data payload, or the final data payload is larger than it should be), the host considers it an error and aborts the transfer.

====== Bus Errors ======
In the event of a bus error or anomaly, an endpoint may receive a SETUP packet in the middle of a control transfer.  In such a case, the endpoint must abort the current transfer and handle the unexpected SETUP packet.  This behavior should be completely transparent to the host; the host should neither expect nor take advantage of this behavior.

====== Halt Conditions ======
A control endpoint may recover from a halt condition upon receiving a SETUP packet.  If the endpoint does not recover from a SETUP packet, it may need to be recovered via a different pipe.  If an endpoint with the endpoint number 0 does not recover with a SETUP packet, the host should issue a device reset.

==== Bulk Data Transfers ====
A pipe with a bulk transfer type provides:
* Access to the USB on a bandwidth-available basis
* Retry of transfers that encounter the occasional delivery failure
* Guaranteed data integrity, but no guaranteed bandwidth

The host controller gives bulk data transfers low priority; they are generally only processed when bandwidth is available, however software may not assume that a control transfer will be processed before a bulk transfer.  If multiple bulk transfers are pending, the host controller may begin moving bulk transfers over the bus according to an implementation-dependent policy.  The system software may vary the bus time made available for a bulk transfer to a specific endpoint.  

The USB does not impose any structure on the data content of a bulk transfer; thus, bulk transfers are carried via stream pipes.

===== Maximum Data Payload Size =====
An endpoint used for a bulk data transfer specifies the '''maximum data payload size''' that it can accept or transmit to the bus.  The allowable maximum data payload sizes depend on the speed of the device:
* High-sped device endpoints may only select a maximum data payload size of 512 bytes.
* Full-speed device endpoints may select a maximum data payload size of 8, 16, 32, or 64 bytes.
* Low-speed devices may not implement bulk endpoints.

Like control transfers, a bulk transfer endpoint must transmit data payloads of the maximum data payload size for that endpoint with the exception of the last data payload in a particular transfer.  The last data payload need not (and should not) be padded out to the maximum data payload size.

The bulk transfer is considered complete when the endpoint has transferred exactly as much data as expected, the endpoint transfers a packet with a data payload size less than the endpoint's maximum data payload size, or the endpoint transfers a zero-length packet.

===== Transmission Errors =====
If a data payload is transferred that is larger than expected, the transfer should be aborted along with any pending bulk transfers through the same pipe.

Bulk data transfers employ data toggle bits to both detect errors and provide the necessary synchronization to recover from an error.  If a halt condition is detected, any remaining bulk transfers should be retired.  The halt condition is resolved by means of a separate control pipe.

==== Interrupt Data Transfers ====
Interrupt data transfers guarantee a maximum service time for any data transfer.  In the even of a transmission failure, data is retransmitted at the next period.  Thus, an interrupt data transfer is ideal for devices that do not send data often, but when they do, they require timely transmission as well as data integrity; most human input devices have these requirements.

Interrupt data transfers are carried out by a stream pipe and thus do not need to adhere to any USB data structure.

===== Maximum Data Payload Size =====
An endpoint used for a interrupt data transfer specifies the '''maximum data payload size''' that it can accept or transmit to the bus.  The allowable maximum data payload sizes depend on the speed of the device:
* High-speed device endpoints may select a maximum data payload size of up to 1024 bytes.
* Full-speed device endpoints may select a maximum data payload size of up to 64 bytes.
* Low-speed device endpoints may select a maximum data payload size of up to 8 bytes.

Additionally, a high-speed, high-bandwidth endpoint may specify that it requires two or three transactions per microframe.  High-speed, high-bandwidth endpoints, frames, and microframes will be discussed later.

Notice that the maximum data payload size for interrupt data transfers allows for more granularity than control or bulk data transfers.  That is, an interrupt data transfer endpoint for a high-speed device may be any integer from 0 to 1024.  The maximum data payload size for an interrupt transfer endpoint remains constant during the lifetime of the device's configuration.

Like control and bulk data transfers, an interrupt transfer endpoint must transmit data payloads of the maximum data payload size for that endpoint with the exception of the last data payload in a particular transfer.  The last data payload need not (and should not) be padded out to the maximum data payload size.

The interrupt transfer is considered complete when the endpoint has transferred exactly as much data as expected, the endpoint transfers a packet with a data payload size less than the endpoint's maximum data payload size, or the endpoint transfers a zero-length packet.

===== Transmission Errors =====
If a data payload is transferred that is larger than expected, the transfer should be aborted and the pipe stalls any future interrupt transfers until the error is acknowledged and corrected.

Interrupt data transfers may use one of two data toggle bit schemes to ensure successful data transmission.  Devices that require higher through-put may choose to toggle every transmission rather than perform a handshake with the host.  This method is more susceptible to errors than the alternative method of toggling bits upon successful transaction (after a handshake).

If a halt condition is detected, any pending interrupt transfers should be retired.  The halt condition is resolved via a separate control pipe.

==== Isochronous Data Transfers ====
Isochronous data transfers are similar to interrupt transfers in that they guarantee a maximum service time for any transfer, but isochronous data transfers do not ensure data integrity.  When data is ready to be transmitted to or from an isochronous endpoint, the data is always transferred at a constant rate.

The data being transmitted via an isochronous pipe need not have any specific structure, therefore isochronous pipes are stream pipes.

===== Maximum Data Payload Size =====
An endpoint used for a isochronous data transfer specifies the '''maximum data payload size''' that it can accept or transmit to the bus.  The allowable maximum data payload sizes depend on the speed of the device:
* High-speed device endpoints may select a maximum data payload size of up to 1024 bytes.
* Full-speed device endpoints may select a maximum data payload size of up to 1023 bytes.
* Low-speed devices may not implement isochronous endpoints.

Like interrupt endpoints, isochronous endpoints may specify a maximum data payload size with byte granularity.  Also like interrupt endpoints, high-speed, high-bandwidth isochronous endpoints may specify if they require two or three transactions per microframe.

Unlike any other transfer types, isochronous transfers may transmit any amount of data up to the maximum data payload size during any transaction.

===== Transmission Errors =====
Isochronous transfers are meant for devices where data transmission rate is more important than data integrity.  For that reason, isochronous transfers do not allow handshakes and thus cannot stall.  It is still important that the agent of an isochronous transfer know if an error occurred, and possibly how much data was lost.  The USB protocol provides several mechanisms for detecting data transmission errors in an isochronous transfer, these mechanisms will be discussed later.  Determining the amount of data lost is implementation-dependent.  It is up to the software on the host or firmware on the function to implement any sort of data corruption detection/correction.

== Advanced USB Concepts ==
The topics in this section build upon the topics previously discussed.  The information in this section provides some useful lower-level details about USB systems.

=== Distribution of Bus Access Time ===

==== Frames and Microframes ====
To ensure synchronization between the host and the functions, the USB divides bus time into fixed-length segments.  For low- or full-speed buses, the USB divides the bus time into 1 millisecond units, called '''frames'''.  For a high-speed bus, the USB divides the bus time into 125 microsecond units, called '''microframes'''.

Note that frames and microframes do not coexist on one bus; low- and full-speed buses used frames, but in developing a high-speed bus, a shorter frame was necessary because the significantly higher signaling bit rate is more sensitive to smaller shifts in synchronization between the host and the function.

Frames and microframes are mostly a physical-layer detail and should not be confused with any of the previous concepts.  Frames and microframes do not correspond to any packet or transaction; in fact, several transactions usually take place during one (micro)frame.  The host controller issues a '''start-of-frame''' ('''SOF''') packet at the beginning of every (micro)frame.  The remainder of the (micro)frame is available for the host controller to carry out transactions.  A transaction may not take place if it cannot be completed in the same (micro)frame (because otherwise the next SOF packet would interrupt the transaction).

[[Image:Usbframes.gif|frame|center|Figure 10: Illustration of USB (micro)frames.]]

It is important to realize that the host controller may rearrange transactions to make better use of the available bandwidth.  Of course, two transactions through the same pipe must occur in the correct order, but the transactions of two separate transfers may be reordered at the host controller's discretion.  Consider a pending bulk transfer and two pending control transfers.  The host could potentially reorder the transfers on the bus as in Figure 11.

[[Image:Usbtransactionreorder.gif|frame|center|Figure 11: Illustrates how a host controller may potentially reorder a bulk transfer and two control transfers on the USB.]]

==== Bus Time Rationing ====
There are separate rules for the allocation of frames on a full-/low-speed bus, and for the allocation of microframes on a high-speed bus.

For full- or low- speed buses:
* If a control transfer requires less than 10% of a frame, the remaining bus time can be used to support bulk transfers
* If there are more control transfers than reserved time, yet additional frame time that is unused by interrupt or isochronous transfers, the host controller may move additional control transfers onto the bus.
* No more than 90% of a frame may be allocated for periodic (isochronous and interrupt) transfers.
* The host must not issue more than 1 transaction in a single frame for a specific isochronous endpoint.

For a high-speed bus:
* If a control transfer requires less than 20% of a microframe, the remaining bus time can be used to support bulk transfers.
* If there are more control transfers than reserved time, yet additional microframe time that is unused by interrupt or isochronous transfers, the host controller may move additional control transfers onto the bus.
* No more than 80% of a frame may be allocated for periodic (isochronous and interrupt) transfers.
* The host must not issue more than 1 transaction in a single microframe for a specific isochronous endpoint unless it is a high-speed, high-bandwidth endpoint.
* Split transaction bus access time is allocated from the 80% of the microframe guaranteed to periodic transfers.

=== High-Speed, High-Bandwidth Endpoints ===
High-speed interrupt or isochronous endpoints that require high bandwidth may specify that they support up to three transaction in a single (micro)frame.  In this case, all but the last transaction in a particular (micro)frame must have a data payload of the maximum data payload size for that endpoint.

The host controller never retries a transaction with an isochronous endpoint.  If a transaction with a high-speed, high-bandwidth interrupt endpoint fails, the host controller may retry the transaction during the same (micro)frame if the maximum number of transactions per (micro)frame has not been reached.  Otherwise, the transaction is retried at the next period.

=== Supporting Isochronous Transfers ===
Recall that isochronous transfers occur over stream pipes, which provide one-way data transfer.  On one of the pipe, called the '''source''', data is produced, and on the other end, called the '''sink''', data is delivered.

Devices that implement isochronous endpoints require that data be transmitted from source to sink at a certain rate, sometimes in large payloads (e.g, streaming audio or video).  This section discusses how the USB accomplishes these requirements.

==== Synchronization ====
Due to application-specific sampling rates, different hardware clock designs, scheduling policies in the operating system, or even physical anomalies, the host and isochronous device could fall out of synchronization.  Therefore, special consideration is required to maintain synchronization.  Isochronous endpoints specify one of three synchronization types.

===== Asynchronous Endpoints =====
'''Asynchronous''' endpoints are incapable of synchronizing to SOF packet frequency (1ms periods for full-speed endpoints, 125 microsecond periods for high-speed endpoints).  These endpoints have either:a set of one or more fixed data sampling rates, or a continuously programmable data rate.  The device must report the programmability of an asynchronous endpoint in some manner (defined by the class of the device rather than by the USB specifications); if the data rate is programmable, then it must be set by the host during initialization of the isochronous endpoint.

Asynchronous source endpoints imply their data rate by the number of samples produced per (micro)frame.  Asynchronous sink endpoints must provide explicit feedback to the source endpoint.  When the source endpoint is the host, it is the responsibility of the device driver to process the explicit feedback properly.  This feedback allows the host and device to make slight adjustments to the data rate in order to compensate for any clock drift.

===== Synchronous Endpoints =====
'''Synchronous''' endpoints must synchronize their data transmissions to the SOF packet frequency (1ms periods for full-speed endpoints, 125 microsecond periods for high-speed endpoints).  These endpoints have either a set of one or more fixed data sampling rates, or a continuously programmable data rate.  The device must report the programmability of a synchronous endpoint in some manner (defined by the class of the device rather than by the USB specifications); if the data rate is programmable, then it must be set by the host during initialization of the isochronous endpoint.

===== Adaptive Endpoints =====
'''Adaptive''' endpoints can source or sink data at any rate within their specified operating range.  These endpoints may have an operating range that centers around a specific data rate, it may have a finite set of data rate ranges, or it may select between several programmable or auto-detecting data rates.  The device must report the programmability of an adaptive endpoint in some manner (defined by the class of the device rather than by the USB specifications); unlike the previous synchronization types, adaptive endpoints may adjust it's instantaneous data rate during operation.

Adaptive sink endpoints provide explicit feedback to the source like asynchronous endpoints.

==== Handling Errors ====
Handshakes are not performed for isochronous transactions, therewith eliminating the bandwidth overhead of acknowledgment packets.  Unlike other transfer types, the applications of isochronous endpoints are responsible for any error detection and handling.  Although it may be more important to continue delivering streaming data rather than retransmit a missed data packet, applications of isochronous endpoints often still need to know that an error did occur in the stream.

The USB protocol highlights the following possible method for the host or a device to detect an error in an isochronous stream:
* High-speed, high-bandwidth isochronous transactions use data PID sequencing (data bit toggling), an isochronous sink can determine that a data packet was missed when it receives an invalid data PID sequence.
* The host controller and device can both see SOF packets on the bus.  If the SOF packet is issued for a (micro)frame that is expected to carry the periodic data of an isochronous endpoint, but the data is not transmitted, then the hardware can determine that a packet was missed.
* The protocol provides CRC protection to ensure that the data has not been corrupted.
* If an endpoint sees the token packet but does not see the associated data packet within a bus transaction timeout period, then the data packet failed to transfer.

Once an application is aware that there is an error in the stream, it is up to the application to determine the next course of action.

== USB Protocol ==

=== Packets ===
The atomic unit of data transfer is a packet.  A packet is a bundle of organized data which typically contains three elements:
* Control information (e.g. source, destination, length of data)
* User/Application-specific data
* Error detection and correction bits

==== SYNC Field ====
The SYNC field is omitted from packet diagrams in the USB specifications, and usually in other material on USB meant for programmers.  Here I will briefly describe the semantics of the SYNC field simply because USB sources often reference the SYNC field which may confuse the reader.  However, for clarity, the system programmer (and probably most USB device firmware developers as well) does not need to know about the SYNC field.

All USB packets start with a SYNC field which serves, unsurprisingly, as a synchronization mechanism between the receiver and the sender.  The SYNC field consists of 6 or 30 alternating bits for low- and full-speed or high-speed buses, respectively.  The last two bits of the SYNC field are equal (and low).  High-speed hubs may drop up to 4 bits of the SYNC field, so a receiving device may not see the entire field, but the final two bits are all the device needs to identify exactly where the SYNC field ends, and useful data begins.

==== Packet Identifier Field ====
The '''Packet Identifier''' ('''PID''') immediately follow the SYNC field.  There are a total of 17 defined PIDs (included the PID of 0000b, which is reserved), therefore a PID requires 4 bits to encode.  If errors on the bus alter the PID field (changing an OUT PID to an IN PID, for example), the result could be anything from unexpected behavior to massive data loss.  Due to the importance of the PID integrity, the PID field is 8 bits wide.  The last 4 bits simply compliment the first four bits, this provides a means to determine if an error on the bus has altered the PID field.  The PID Field is illustrated below.
<center>
{| border="1" cellpadding="15"
|+ align="bottom" style="font-weight:bold;" |Packet Identifier Field Format
|-
|PID<span style="vertical-align:sub;">0</span>
|PID<span style="vertical-align:sub;">1</span>
|PID<span style="vertical-align:sub;">2</span>
|PID<span style="vertical-align:sub;">3</span>
|style="text-decoration:overline;" |PID<span style="vertical-align:sub;">0</span>
|style="text-decoration:overline;" |PID<span style="vertical-align:sub;">1</span>
|style="text-decoration:overline;" |PID<span style="vertical-align:sub;">2</span>
|style="text-decoration:overline;" |PID<span style="vertical-align:sub;">3</span>
|}
</center>

PID codes are categorized into 4 groups which share the same two least-significant bits.  USB 2.0 defines the PIDs in the following table.

{| border="1" cellpadding="5"
|+ align="bottom" style="font-weight:bold;" |PID Types
|-
!style="white-space:nowrap;" |PID Type
!style="white-space:nowrap;" |PID Name
!style="white-space:nowrap;" |PID [3:0]
!style="white-space:nowrap;" |Description
|-
|rowspan="4" align="center" |Token
|align="center" |OUT
|align="center" |0001b
|The packet describes a host-to-function transaction.
|-
|align="center" |IN
|align="center" |1001b
|The packet describes a function-to-host transaction.
|-
|align="center" |SOF
|align="center" |0101b
|The packet marks the start of frame and specifies the frame number.
|-
|align="center" |SETUP
|align="center" |1101b
|Packet describes a SETUP transaction from the host to the function via a control pipe.
|-
|rowspan="4" align="center" |Data
|align="center" |<div id="DATAPIDS">DATA0</div>
|align="center" |0011b
|This packet is an even data packet.
|-
|align="center" |DATA1
|align="center" |1011b
|This packet is an odd data packet
|-
|align="center" |DATA2
|align="center" |0111b
|This packet is only used in high-speed, high-bandwidth isochronous transfers.
|-
|align="center" |MDATA
|align="center" |1111b
|This packet is only used in split transactions, or high-speed, high-bandwidth isochronous transfers.
|-
|rowspan="4" align="center" |Handshake
|align="center" |ACK
|align="center" |0010b
|This packet acknowledges the successful receipt of a data packet.
|-
|align="center" |NAK
|align="center" |1010b
|This packet indicates that data is not ready to be transmitted yet.
|-
|align="center" |STALL
|align="center" |1110b
|This packet indicates that the endpoint has halted, or a control pipe does not support a certain request.
|-
|align="center" |NYET
|align="center" |0110b
|The receiver has not yet responded, or the host should begin sending PING packets.
|-
|rowspan="5" align="center" |Special
|align="center" |PRE
|align="center" |1100b
|This packet is a host-issued preamble for a split-transaction.
|-
|align="center" |ERR
|align="center" |1100b
|This packet is a handshake response that a split transaction error occurred.  Note that this PID is identical to the PID for a PRE packet.
|-
|align="center" |SPLIT
|align="center" |1000b
|This packet supports split transactions between the host and a high-speed hub.
|-
|align="center" |PING
|align="center" |0100b
|This packet is used for flow-control in high-speed control and bulk transfers.
|-
|align="center" |Reserved
|align="center" |0000b
|This is a reserved PID and must not be used.
|}

==== Address Fields ====
Address fields select a specific endpoint on a specific function.  Naturally, two such fields are defined: an address field and an endpoint field.  All devices must fully decode these fields; a mismatch of either field (including an endpoint field which specifies an endpoint that have not been initialized) must be ignored.

===== Address Field =====
The address field is specified for the following PIDs:
* IN
* SETUP
* OUT
* PING
* SPLIT

The address field is 7 bits wide and illustrated below.  Each possible value may only indicate a single function.  Address zero is reserved as the '''default address''' and cannot be assigned to any function.  All functions must respond to the default address upon reset and power-up until the host assigns the function a specific address.  Therefore, one host controller can support up to 127 devices at one time.
<center>
{| border="1" cellpadding="15"
|+ align="bottom" style="font-weight:bold;" |Address Field
|-
|Addr<span style="vertical-align:sub;">0</span>
|Addr<span style="vertical-align:sub;">1</span>
|Addr<span style="vertical-align:sub;">2</span>
|Addr<span style="vertical-align:sub;">3</span>
|Addr<span style="vertical-align:sub;">4</span>
|Addr<span style="vertical-align:sub;">5</span>
|Addr<span style="vertical-align:sub;">6</span>
|}
</center>

===== Endpoint Field =====
The endpoint field is specified for the following PIDs:
* IN
* SETUP
* OUT
* PING

The endpoint field is 4 bits wide and illustrated below.  All functions must support at least endpoint number 0 (the default control pipe).  Low-speed functions may only implement 2 additional pipes, while full- and high-speed devices are only limited by the width of the endpoint field.  In other words, the width of the endpoint field is the reason that full- and high-speed devices are limited to implementing up to 15 additional IN endpoints, and 15 additional OUT endpoints, as noted above under [[#Device_Endpoints_and_Endpoint_Numbers|Device Endpoints and Endpoint Numbers]].
<center>
{| border="1" cellpadding="15"
|+ align="bottom" style="font-weight:bold;" |Endpoint Field
|-
|Endp<span style="vertical-align:sub;">0</span>
|Endp<span style="vertical-align:sub;">1</span>
|Endp<span style="vertical-align:sub;">2</span>
|Endp<span style="vertical-align:sub;">3</span>
|}
</center>

==== Data Field ====
The data field may range from zero to 1,024 bytes and must be an integral number of bytes.  Data bytes are sent least-significant-bit first.

==== Cyclic Redundancy Checks ====
Cyclic Redundancy Checks (CRC) protect all non-PID fields and provide 100% coverage for all single- and double-bit errors.  CRCs are provided for each token field as well as the data field.  This provides a mechanism for the host or device to recognize and either correct or ignore corrupted fields or, in most cases, an entire corrupted packet.

=== Handshakes ===
Transaction types which support flow control return handshakes to indicate:
* Successful reception of data
* Command acceptance or rejection
* Flow control
* Halt conditions

Handshakes are always returned in the handshake phase of a transaction, but may also be returned in the data phase (in place of an expected data packet).  To best understand a certain handshake response, it is useful to understand what each handshake packet type means, as well as the conditions under which each handshake response may be issued.  This section is divided thusly.

==== Handshake Packets ====
All of the handshake packet types were listed previously and briefly in [[#Packet_Identifier_Field|Packet Identifier Field]].  This section discusses those packet types in greater detail.

===== ACK =====
{| style="float:right; border: 1px solid #404040; clear:right;" cellspacing="2"
|+ style="font-weight:bold;" |ACK Handshake Packet
!style="white-space:no-wrap; border-bottom: 1px solid #404040;" |May be issued by...
!style="white-space:no-wrap; border-bottom: 1px solid #404040;" |For these transactions
|-
|align="center" style="border-right: 1px solid #000000;" |Host
|align="center" |IN
|-
|align="center" style="border-right: 1px solid #000000; border-top: 1px solid #000000;" rowspan="3" |Function
|align="center" style="border-top: 1px solid #000000;" |OUT
|-
|align="center" |SETUP
|-
|align="center" |PING
|}
An ACK handshake is issued to communicate that a data packet was successfully received without any bit stuffing or CRC errors over the data field, and the PID field was not corrupted.

ACK packets may be issued when the receiver's sequence bit matches the sequence bit of the received data packet (and the data can be accepted), but the an ACK packet may also be issued when the receiver's sequence bit does not match the sequence bit of the received data packet (and the data cannot be accepted).  This may seem counter intuitive, but the reasoning will become clear in the sections discussing data toggling.

===== NAK =====
{| style="float:right; border: 1px solid #404040; clear:right;" cellspacing="2"
|+ style="font-weight:bold;" |NAK Handshake Packet
!style="white-space:no-wrap; border-bottom: 1px solid #404040;" |May be issued by...
!style="white-space:no-wrap; border-bottom: 1px solid #404040;" |For these transactions
|-
|align="center" style="border-right: 1px solid #000000;" rowspan="3" |Function
|align="center" |IN
|-
|align="center" |OUT
|-
|align="center" |PING
|}
The NAK handshake packet is generally used for flow control to indicate that a function is temporarily unable to transmit or receive data.  The host never issues a NAK handshake packet to a device.

A function returns a NAK handshake packet to the host after an OUT transaction when the function is unable to receive data (usually because the function's internal buffer is currently full).  This response is not an error, but instead indicates that the host should retry transmission later, allowing the function time to process the data currently in its buffer.

A function returns a NAK handshake packet to the host during the data phase of an IN transaction to indicate that the function does not have any data to transfer.

===== STALL =====
{| style="float:right; border: 1px solid #404040; clear:right;" cellspacing="2"
|+ style="font-weight:bold;" |STALL Handshake Packet
!style="white-space:no-wrap; border-bottom: 1px solid #404040;" |May be issued by...
!style="white-space:no-wrap; border-bottom: 1px solid #404040;" |For these transactions
|-
|align="center" style="border-right: 1px solid #000000;" rowspan="3" |Function
|align="center" |IN
|-
|align="center" |OUT
|-
|align="center" |PING
|}
A function uses the STALL handshake packet to indicate that it is unable to transmit or receive data.  Besides the default control pipe, all of a function's endpoints are in an undefined state after the device issues a STALL handshake packet.  The host must never issue a STALL handshake packet.

Typically, the STALL handshake indicates a functional stall.  A '''functional stall''' occurs when the ''halt feature'' (which will be covered under "USB Framework") of an endpoint is set.  In this circumstance, host intervention is required via the default control pipe to clear the ''halt feature'' of the halted endpoint.

Less often, the function returns a STALL handshake during a SETUP or DATA stage of a control transfer.  This is called a '''protocol stall''' and is resolved when the host issues the next SETUP transaction.
{| style="float:right; border: 1px solid #404040; clear:right;" cellspacing="2"
|+ style="font-weight:bold;" |NYET Handshake Packet
!style="white-space:no-wrap; border-bottom: 1px solid #404040;" |May be issued by...
!style="white-space:no-wrap; border-bottom: 1px solid #404040;" |For these transactions
|-
|align="center" style="border-right: 1px solid #000000;" |Hub
|align="center" |SPLIT
|-
|align="center" style="border-right: 1px solid #000000; border-top: 1px solid #000000;" |Function
|align="center" style="border-top: 1px solid #000000;" |OUT
|}
===== NYET =====
The NYET packet may be issued by a function as part of the PING protocol.

Hubs may issue a NYET handshake packet in response to a split transaction that has not yet completed on the low-/full-speed bus.

{| style="float:right; border: 1px solid #404040; clear:right;" cellspacing="2"
|+ style="font-weight:bold;" |ERR Handshake Packet
!style="white-space:no-wrap; border-bottom: 1px solid #404040;" |May be issued by...
!style="white-space:no-wrap; border-bottom: 1px solid #404040;" |For these transactions
|-
|align="center" style="border-right: 1px solid #000000;" |Hub
|align="center" |SPLIT
|}
===== ERR =====
Hubs may issue the special ERR handshake packet to report an error on a low-/full-speed bus as part of the split transaction protocol.

==== Function/Host Response Circumstances ====
This section describes the functional circumstances that cause the host or a function to issue an expected response, no response, or certain handshake packet responses.  The tables in this section are taken and slightly modified for clarity from the USB 2.0 specifications, section 8.4.6.  Dashes denote a "don't care."

===== Function Response to IN Transactions =====
{| border="1" cellpadding="3"
!Token received corrupted
!Function Tx endpoint halt feature
!Function can transmit data
!Action taken by function
|-
|align="center" |Yes
|align="center" | -
|align="center" | -
|Return no response
|-
|align="center" |No
|align="center" |Set
|align="center" | -
|Issue STALL handshake
|-
|align="center" |No
|align="center" |Not Set
|align="center" |No
|Issue NAK handshake
|-
|align="center" |No
|align="center" |Not Set
|align="center" |Yes
|Issue data packet
|}


===== Host Response to IN Transactions =====
{| border="1" cellpadding="3"
!Data packet corrupted
!Host can accept data
!Action taken by host
!Handshake returned by host
|-
|align="center" |Yes
|align="center" | -
|Discard Data
|Return no response
|-
|align="center" |No
|align="center" |No
|Discard Data
|Return no response
|-
|align="center" |No
|align="center" |Yes
|Accept Data
|Issue ACK handshake
|}


===== Function Response to OUT Transactions =====
{| border="1" cellpadding="3"
!Data packet corrupted
!Receiver halt feature
!Sequence bits match
!Function can accept data
!Action taken by function
|-
|align="center" |Yes
|align="center" | -
|align="center" | -
|align="center" | -
|Return no response
|-
|align="center" |No
|align="center" |Set
|align="center" | -
|align="center" | -
|Issue STALL handshake
|-
|align="center" |No
|align="center" |Not Set
|align="center" |No
|align="center" | -
|Issue ACK handshake
|-
|align="center" |No
|align="center" |Not Set
|align="center" |Yes
|align="center" |Yes
|Issue ACK handshake
|-
|align="center" |No
|align="center" |Not Set
|align="center" |Yes
|align="center" |No
|Issue NAK handshake
|}


===== Function Response to SETUP Transactions =====
A function must always accept data in a SETUP transaction, and must never issue a STALL or NAK handshake in response.  All non-control endpoints must simply ignore any SETUP transaction addressed to that endpoint.  This allows SETUP transactions to function as a (re)synchronization mechanism between the host and a function's control endpoint.

=== PING Transaction Protocol ===
Consider a USB mass storage device.  During a transfer from the host to the function, the function's buffer fills up with data that is pending being committed to the physical media.  When the function's buffer is full, the function cannot accept new data until some of the buffer is committed, so if the host continues sending OUT transactions, the function must [[#NAK|NAK]] them.

The problem with this OUT/NAK model is that a function must wait for the handshake stage of the OUT transaction before responding with a NAK.  Since the handshake stage occurs after the data stage, this can waste a significant amount of bandwidth.  Low- and full-speed buses suffer from this problem, but the USB 2.0 specification introduced the PING transaction protocol for high-speed buses.

The PING transaction protocol is very straightforward.  Rather than an OUT transaction, the host issues a PING transaction to the function when the host wishes to send data.  The function responds with either NAK to indicate that it is not ready to receive data (specifically, the function's buffer cannot accommodate the endpoint's maximum data payload amount of data), or ACK to indicate that the host may start sending data.

The USB 2.0 framework allows endpoints to specify an interval, in terms of microframes, which is the amount of microframes that the host should wait before issuing another PING packet to the endpoint.  However, the host is not required to wait this interval before issuing the next PING packet.

During a high-speed control or bulk transfer from the host to function, when an OUT transactions causes a function's free buffer space to drop below the endpoint's maximum data payload, the function responds with a NYET handshake packet.  This indicates that the host should start issuing PING packets rather than additional OUT transactions.

=== Data Toggle Synchronization ===
During a transfer, the host and function must remain synchronized.  The ability to maintain synchronization means that the host or function can detect when synchronization has been lost and, in most cases, resynchronize.

Every endpoint maintains, internally (in the function's hardware), a data toggle bit, also called a data sequence bit.  The host also maintains a data toggle bit for every endpoint with which it communicates.  The state of the data toggle bit on the sender is indicated by which [[#DATAPIDS|DATA PID]] the sender uses.

The receiver toggles its data sequence bit when it is able to accept data and it receives an error-free data packet with the expected [[#DATAPIDS|DATA PID]].  The sender toggles its data sequence bit only upon receiving a valid ACK handshake.  This data toggling scheme requires that the sender and receiver synchronize their data toggle bits at the start of a transaction.

Data toggle synchronization works differently depending on the type of transfer used:
* Control transfers initialize the endpoint's data toggle bits to 0 with a SETUP packet.
* Interrupt and Bulk endpoints initialize their data toggle bits to 0 upon any configuration event.
* Isochronous transfers do not perform a handshake and thus do not support data toggle synchronization.
* High-speed, high-bandwidth isochronous transfers do support data sequencing within a microframe.


The remainder of this section illustrates how the sending and receiving devices each manage their data toggle bits during different transmission scenarios.  Black arrows signify the intended data transmission on the USB.  Gray arrows signify that the intended data transmission completed without error.  Red, discontinuous arrows signify that the intended data was corrupted during transmission or entirely failed to transmit.

==== Successful transmissions ====
[[Image:SuccessfulDataTx.gif|thumb|right|Figure 12: Illustration of how the sender and receiver manage their data toggle bits during a successful data transfer]]
Figure 12 illustrates a successful data transfer.  Both devices have data toggle bits set to 0 at the beginning of transfer i.  Accordingly, the sending device issues a DATA0 PID followed by the data packet.  The receiving device successful reads the DATA0 PID as well as the data packet.  Since the receiver's data toggle bit matches the DATA0 PID and there were no errors in transmitting the remaining data, the receiver toggles its data toggle bit to 1 and issues an ACK handshake response.  The sender receives the ACK handshake without error, and thus toggles its data toggle bit to 1.

Supposing that the next transfer occurs without error as well, the only difference is that the DATA1 PID is used rather than DATA0, and the sending and receiving devices toggle their data toggle bits from 1 to 0 in the same stages that the same bit toggled to a 1 in the previous transfer.

[[Image:FailedDataTx.gif|thumb|left|Figure 13: Illustration of how the sender and receiver manage their data toggle bits during a failed or corrupt data transfer]]
==== Failed or corrupted data transmissions ====
Figure 13 illustrates a failed or corrupted data transmission.  Both devices have data toggle bits set to 0 at the beginning of transfer i.  Accordingly, the sending device issues a DATA0 PID followed by the data packet.  The receiving device either does not see the data packet, or reads a corrupted data packet.  The receiver maintains its data toggle bit and issues a NAK handshake.  The sender successfully sees the NAK handshake and thus does not toggle its data toggle bit.

At the beginning of the next transfer, both the sending and receiving device have data toggle bits still set to 0.  Supposing this transfer completes successfully, it is carried as as described above, under [[#Successful_transmissions|successful transmissions]].

==== Failed or corrupted ACK handshake ====
[[Image:FailedACKTx.gif|thumb|right|Figure 14: Illustration of how the sender and receiver manage their data toggle bits during a failed or corrupt ACK response]]
Figure 14 illustrates a failed or corrupted ACK handshake.  Both devices have data toggle bits set to 0 at the beginning of transfer i.  Accordingly, the sending device issues a DATA0 PID followed by the data packet.  The receiving device successfully reads the DATA0 PID as well as the data packet.  Since the receiver's data toggle bit matches the DATA0 PID and there were no errors in transmitting the remaining data, the receiver toggles its data toggle bit to 1 and issues an ACK handshake response.  The sender does not receive, or receives a corrupted ACK response, and thus discards the packet without modifying it's data toggle bit.

At this point, the sending device's data toggle bit is still 0, and the receiving device's data toggle bit has been set to 1.  The sender, having not seen a valid ACK response for transfer i, reattempts transfer i.  With a data toggle bit of 0, the sender issues a DATA0 PID followed by the data packet.  The receiving device successfully reads the DATA0 PID as well as the data packet.  Since the receiver's data toggle bit does not match the DATA0 PID, the receiver maintains it's data toggle bit value of 1 and issues an ACK handshake response.  The sender receives the ACK response without error, and thus toggles its data toggle bit to 1.

Supposing that the next transfer occurs without error, it begins with both device's data toggle bits set to 1 and ends with them toggling to 0 at the appropriate stage of the transfer.

=== USB Transfers Revisited ===
A lot of information has been introduced since [[#Basics_of_USB_Transfers|Basics of USB Transfers]], and it is very easy to get lost in the details.  With even a decent understanding of the four types of USB transfers, it is often difficult to extrapolate from the intricacies of the USB protocol to an understanding of just how everything fits together.  For these reasons, this section intends to clarify some potentially confusing concepts both explicitly and implicitly by revisiting the four transfer types in context of all the information covered since first discussing them.

An apprehensive reader may have noticed that some terms like SETUP and DATA are used both in referring to [[#Packet_Identifier_Field|packet identifiers]], and in referring to types of transactions.  This wiki entry may very well be the first and only source of USB information that takes a moment to specifically differentiate between the two.

Under [[#Basics_of_USB_Transfers|Basics of USB Transfers]], USB transactions were mentioned only briefly as has been reproduced below:
 Most USB transactions consist of three packets:
 * A '''token packet''' indicates the type and direction of the transaction, the device address, and an
   endpoint number.
 * Depending on the direction of the transaction, either the host or the function sends a '''data packet'''
   (which may simply indicate that there is no data to send).
 * The receiving device responds with a '''handshake packet''' to indicate if the transfer was successful.

Then, under [[#Packets|Packets]], a packet was described as "the atomic unit of data transfer."

If a packet is an atom, then a transaction would be a molecule.  That is, a transaction is made up of several packets in a specific order, and the packets which make up a transaction cannot be reordered or separated and still yield the same transaction.  Transactions are normally named after their token packet (or their "special" packet, in the case of PING or SPLIT because these special packets play the same role as token packets), with the exception that IN or OUT transactions are often referred to, collectively, as DATA transactions.  In examples, transactions that contain a data stage often indicate the type of DATA PID used by either appending 0, 1, 2, or M to the name, or adding it in parenthesis (e.g, SETUP(0) or SETUP0, OUT1 or OUT(1)).

[[Image:SETUPTransaction.gif|frame|center|Figure 15: Illustrates a SETUP transaction, which consists of a SETUP packet, a DATA0 packet, and a handshake packet (in this case, an ACK handshake).]]
An example of a single SETUP transaction is depicted in figure 15.  This transaction contains the typical three packets.  The token packet has a SETUP PID, the data packet has a DATA0 PID (recall that a SETUP packet initializes both the function's and the host's data toggle bits to 0), and the handshake response has an ACK PID.

Transfers are made up of transactions.  Transactions may not be reordered within a transfer but, as discussed in [[#Frames_and_Microframes|Frames and Microframes]], the transactions of a particular transfer may or may not be sent over the bus in a continuous fashion.  The rest of this section looks at the transactions involved in the four transfer types.

==== Control Transfers ====
Control transfers are the only transfers that use the SETUP transaction.  Control transfers take place in up to three stages:
* The SETUP stage consists simply of a SETUP transaction
* The DATA stage is optional.  If used, it may contain either one or more IN transactions, or one or more OUT transactions.  The first of these IN or OUT transactions uses the DATA1 PID.  The second, if present, uses the DATA0 PID, the third DATA1, and so on.
* The STATUS stage consists of a single IN or a single OUT transaction, which must use the DATA1 PID.  If the DATA stage is present, then the STATUS stage uses the opposite type of transaction as the DATA stage (i.e, if the DATA stage consists of one or more OUT transactions, the STATUS stage consists of a single IN transaction, and vice versa).  When the DATA stage is omitted, the STATUS stage uses a single IN transaction.

Figure 16 is taken from Figure 8-37 of the USB 2.0 specification and illustrates the transaction order, data sequence bit value, and DATA PID type for control read and write sequences.
[[Image:ControlTx.gif|frame|center|Figure 16: Control read and write sequences]]

==== Bulk and Interrupt Transfers ====
In the context of the USB protocol, the only difference between bulk and interrupt transfers is that bulk transfers, when operating at high-speed, support the [[#PING_Transaction_Protocol|PING Transaction Protocol]].  Note that in a general context, these two transfer types are also different in that they are scheduled differently by the host (refer to [[#Bus_Time_Rationing|Bus Time Rationing]]).

All bulk and interrupt endpoints transfer in one direction.  The data toggle bits for these endpoints are initialized to zero after any configuration event.  Figure 17 is taken from Figure 8-35 of the USB 2.0 specification and illustrates the bulk and interrupt transactions for both IN and OUT endpoints.  Note that, even though the figure only mentions bulk reads and bulk writes, the USB 2.0 specification references the same figure from section 8.5.4, on Interrupt Transactions.
[[Image:BulkInterruptTx.gif|frame|center|Figure 17: Bulk and interrupt transactions]]

==== Isochronous Transfers ====
Isochronous transfers are the only type of transfers whose transactions do not have a handshake phase.  Isochronous transfers should only use DATA0 PIDs, however the host controller must support DATA1 PIDs as well, even though isochronous transfers do not use a data synchronization bit mechanism.

==== High-Speed, High-Bandwidth Isochronous Transfers ====
High-speed, high-bandwidth isochronous transfers are a special case of isochronous transfers, where up to 3 transactions may occur in one microframe.  As a specific type of isochronous transfer, high-speed, high-bandwidth isochronous transfers omit the handshake phase of their transactions.  Since up to 3 transactions may occur in one microframe, high-speed, high-bandwidth isochronous transfers, it is necessary to use a data sequencing mechanism like the other transfer types.

USB 2.0 does implement a data sequencing mechanism for high-speed, high-bandwidth isochronous transfers, but it works a little differently than as in other transfer types.  In fact, data sequencing works differently depending on whether an endpoint is an IN, or an OUT high-speed, high-bandwidth isochronous endpoint.

For IN high-speed, high-bandwidth isochronous endpoints, the data sequencing is depicted in figure 18, which has been taken from figure 5-11 of the USB 2.0 specifications.  The last transaction in a microframe always uses the DATA0 PID.  The second-to-last transaction in a microframe uses the DATA1 PID, and the third-to-last transaction in a microframe always uses the DATA2 PID.
[[Image:HSHBIsoIN.gif|frame|center|Figure 18: Data Phase Sequence for Isochronous IN High Bandwidth Endpoints]]

For OUT high-speed, high-bandwidth isochronous endpoints, the data sequencing is depicted in figure 19, which has been taken from figure 5-12 of the USB 2.0 specifications.  All transactions but the last transaction use the MDATA PID.  The last transaction uses either the DATA0, DATA1, or DATA2 PID, depending on how many transactions were intended to take place during the microframe.  If one transaction was meant to take place, it is also the last transaction and uses a DATA0 PID.  If two transactions were meant to take place, the last transaction uses a DATA1 PID.  If three transactions were meant to take place, the last transaction uses a DATA2 PID.
[[Image:HSHBIsoOUT.gif|frame|center|Figure 19: Data Phase Sequence for Isochronous OUT High Bandwidth Endpoints]]

== USB Device Framework ==
The USB device framework is the thing that makes USB support so appealing.  The transfer types and USB protocol are well-designed, of course, but the USB device framework defines standard device states that all devices must support, as well as standard requests and responses that allow the host to retrieve more than enough information about a device to determine the correct device driver and report information about the device even if the correct device driver isn't available (e.g, the manufacturer's name, the product's name, etc).

=== Functions, Configurations, Interfaces, and Endpoints ===
[[Image:FunctionConfigsIFacesEPs.gif|frame|right|Figure 20: Illustration of the relationship between functions, configurations, interfaces, and endpoints.]]
All USB devices, or functions, have at least one configuration, and every configuration has at least one interface.  An interface may define zero or more endpoints.  This relationship is illustrated in figure 20.

Although configurations descriptors are addressed sequentially starting with configuration descriptor zero, each configuration specifies a unique (within the scope of the function), none-zero configuration value.  The '''configuration value''' is what the host needs to know in order to apply a certain configuration to a device.  When asking for the current configuration of a device, a returned value of zero indicates that the device is not configured and is thus in the address state.

An '''interface''' defines the functional use of a set of endpoints and may imply that certain class-specific requests can be executed via the default control pipe.  Thus, an interface need not necessarily define any additional endpoints.  No interface may define the functional use of endpoint zero.

Each interface describes a unique set of endpoints within the scope of the configuration.  However, an interface may provide one or more '''alternate settings''', which may have different definitions for the same set of endpoints.  When the host selects an alternate setting for an interface, the alternate setting's definitions are used instead of the default settings of the same interface.

=== USB Device States ===
A USB device may define states that are internal to the device, however the USB device framework defines a set of states that are visible to both the host and the device.  Those visible states are the following:

* '''Attached''' - Immediately after the USB device is attached to the USB system, it is in this state.  The USB specifications do not define the state of a USB device that is detached from a USB system.
* '''Powered''' - A device is in this state after it has both been attached to the bus, and the V<span style="vertical-align:sub;">BUS</span> line is applied to the device (the host controller drives the V<span style="vertical-align:sub;">BUS</span> at +5V, however this is only particularly important for hardware developers).  In this state, the device must not respond to any bus transactions.  The USB specification recognizes three potential scenarios with respect to how a device draws power:
** ''Self-Powered Devices'' draw power from an external power source (e.g, a USB printer plugs into the wall as well as a USB port).  Although the device may be considered technically "powered" even before attachment to the USB, it is still only considered powered after the V<span style="vertical-align:sub;">BUS</span> line is applied to the device.
** ''Bus-Powered Devices'' draw power solely from the USB up to 100mA.
** ''Self- or Bus-Powered Devices'' may draw power from either the bus or an external power source, depending on the configuration.  These devices may change power source at any time.  If a device is currently self-powered and requires more than 100mA of power, but switches to being bus-powered, then the device must return to the Address state.
* '''Default''' - A device in the powered state enters the default state after receiving a bus reset.  In this state, the device is addressable at the default, reserved address of 0.  At this point, the device is operating at the correct speed.  The host is expected to allow 10 milliseconds before expecting the device to respond to data transfers after reset.
* '''Address''' - A device enters this state after the host assigns it an address via the default control pipe, which is always accessible whether the device's address has been set or not.
* '''Configured''' - A device is in this state after the host examines its possible configurations and selects one.  All endpoint's data toggle bits are initialized to zero when a device enters this state.
* '''Suspended''' - When no traffic is observed on the bus for a period of 1 millisecond, a USB device enters this state, characterized by its low power consumption.  The device's address and configuration settings are maintained while suspended.  A device exits the suspended state as soon as it begins seeing bus activity again.  The host is expected to allow 10 milliseconds before expecting the device to respond to data transfers after resume.

=== Remote Wakeup Capability ===
One of the reasons a USB device may stop seeing USB traffic and thus enter a suspended state is because the host may have entered a suspended state as well.  Some devices, typically keyboards and mice, support the ability to issue a remote wakeup signal to the host.  In case the host software does not support remote wakeup, this capability must be disabled when a USB device is reset.  If the host does support remote wakeup, then it may selectively enable the remote wakeup capability for specific devices (typically as chosen by the user).  Then, these devices may issue a remote wakeup signal while in a suspended state to request that the host exits its own suspended state.

=== USB Device Enumeration ===
The following describes the process of bus enumeration, which occurs after a device is connected to a powered port:

# The hub to which the device has been attached notifies the host via its status change pipe.  The newly attached device is in the powered state at this point, and the port to which it has been attached is disabled.
# The host queries more information from the hub to determine that a device has been attached, and to which port.
# The host must wait at least 100 millisecond to allow a device to complete its insertion process, and for power to stabilize at the device.  After the delay, the host enables the port and issues a reset signal to the device for at least 50 milliseconds.
# The hub performs any required reset processing.  After the reset signal has been released, the port is enabled and the device enters the default state.
# The host assigns the device a unique address, thereby transitioning the device into the address state.
# The host requests the device descriptor from the device via the default control pipe in order to determine the actual maximum data payload size of the default control pipe for the device.  This step may occur before or after the host assigns the device an address.
# The host reads all the possible device configuration information.
# The host selects a certain configuration from the list of configurations supported by the device and sets the device to use that configuration.  Optionally, the host may also select alternate interface settings within a configuration.  All endpoints are initialized as described by the selected configuration, and the device is ready to use.

=== USB Device Requests ===
Standard, class-specific, and vendor-specific requests are made to the USB device over the default control pipe.  The SETUP transaction always has a data payload size of 8 bytes, as noted in the [[#Maximum_Data_Payload_Size|Maximum Data Payload Size]] section of [[#Control_Transfers|Control Transfers]].  The format of the setup data is as follows:

{| border="1" cellpadding="5"
!Offset
!Field
!Size
!Type
!Description
|- valign="top" align="center"
|0
|bmRequestType
|1
|Bitmap
|align="center" |
{| border="1" cellpadding="15"
|D<span style="vertical-align:sub;">7</span>
|D<span style="vertical-align:sub;">6</span>
|D<span style="vertical-align:sub;">5</span>
|D<span style="vertical-align:sub;">4</span>
|D<span style="vertical-align:sub;">3</span>
|D<span style="vertical-align:sub;">2</span>
|D<span style="vertical-align:sub;">1</span>
|D<span style="vertical-align:sub;">0</span>
|}
{| border="0" cellpadding="0"
|valign="top" style="min-width:80px;" |'''D<span style="vertical-align:sub;">7</span>'''
|'''Data transfer direction''' ''* The value of this bit is ignored when wLength is zero''
* 0b = Host-to-device
* 1b = Device-to-host
|-
|valign="top" style="min-width:80px;" |'''D<span style="vertical-align:sub;">6...5</span>'''
|'''Type of request'''
* 00b = Standard
* 01b = Class
* 10b = Vendor
* 11b = Reserved
|-
|valign="top" style="min-width:80px;" |'''D<span style="vertical-align:sub;">4...0</span>'''
|'''Recipient'''
* 00000b = Device
* 00001b = Interface
* 00010b = Endpoint
* 00011b = Other
* 00100b to 11111b = Reserved
|}
|- valign="top" align="center"
|1
|bRequest
|1
|Value
|align="left" |Specific request
|- valign="top" align="center"
|2
|wValue
|2
|Value
|align="left" |Word-sized field the may (or may not) serve as a parameter to the request, depending on the specific request.
|- valign="top" align="center"
|4
|wIndex
|2
|Index or offset
|align="left" |Word-sized field that may (or may not) serve as a parameter to the request, depending on the specific request.  Typically this field holds an index or an offset value.<br /><br />
When bmRequestType specifies an endpoint as the recipient, the format of this field is as follows:
{| border="1" align="center"
|- align="center"
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">7</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">6</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">5</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">4</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">3</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">2</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">1</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">0</span>'''
|- align="center"
|Direction
|colspan="3" |Reserved (reset to zero)
|colspan="4" |Endpoint Number
|- align="center"
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">15</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">14</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">13</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">12</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">11</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">10</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">9</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">8</span>'''
|- align="center"
|colspan="8" |Reserved (reset to zero)
|}
The direction bit (bit '''D<span style="vertical-align:sub;">7</span>''') is set to zero to indicate the OUT endpoint with the specified endpoint number, or it is set to one to indicate the IN endpoint with the specified endpoint number.  The host should always set the direction bit to zero (but the device should accept either value) when the endpoint is part of a control pipe.

When bmRequestType specifies an interface as the recipient, the format of this field is as follows:
{| border="1" align="center"
|- align="center"
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">7</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">6</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">5</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">4</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">3</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">2</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">1</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">0</span>'''
|- align="center"
|colspan="8" |Interface Number
|- align="center"
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">15</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">14</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">13</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">12</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">11</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">10</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">9</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">8</span>'''
|- align="center"
|colspan="8" |Reserved (reset to zero)
|}
|- valign="top" align="center"
|6
|wLength
|2
|Count
|align="left" |Number of bytes to transfer if there is a DATA stage.<br /><br />
* If this field is non-zero, and bmRequestType indicates a transfer from device-to-host, then the device must never return more than ''wLength'' bytes of data.  However, a device may return less.
* If this field is non-zero, and the bmRequestType indicates a transfer from host-to-device, then the host must send exactly ''wLength'' bytes of data.  If the host sends more than ''wLength'' bytes, the behavior of the device is undefined.
|}

When a device receives a request that is undefined, is inappropriate given the current setting or state of the device, or uses values that are inappropriate for the particular request, then a '''Request Error''' exists.  The device handles a Request Error by returning a STALL PID to the next DATA or STATUS stage, preferably at the next DATA stage transaction.

=== Standard Requests ===
{|border="1" cellpadding="2" style="float:right;margin-left:20px;"
|+align="bottom" |'''Standard USB Request Codes'''
!bRequest
!Value
|-
|[[#GET_STATUS|GET_STATUS]]
|align="center" |0
|-
|[[#CLEAR_FEATURE|CLEAR_FEATURE]]
|align="center" |1
|-
|''Reserved''
|align="center" |2
|-
|[[#SET_FEATURE|SET_FEATURE]]
|align="center" |3
|-
|''Reserved''
|align="center" |4
|-
|[[#SET_ADDRESS|SET_ADDRESS]]
|align="center" |5
|-
|[[#GET_DESCRIPTOR|GET_DESCRIPTOR]]
|align="center" |6
|-
|[[#SET_DESCRIPTOR|SET_DESCRIPTOR]]
|align="center" |7
|-
|[[#GET_CONFIGURATION|GET_CONFIGURATION]]
|align="center" |8
|-
|[[#SET_CONFIGURATION|SET_CONFIGURATION]]
|align="center" |9
|-
|[[#GET_INTERFACE|GET_INTERFACE]]
|align="center" |10
|-
|[[#SET_INTERFACE|SET_INTERFACE]]
|align="center" |11
|-
|[[#SYNC_FRAME|SYNC_FRAME]]
|align="center" |12
|}
The standard requests are defined for all USB devices, and all USB devices must respond to these standard requests even if the device hasn't been assigned an address, or the device hasn't been configured.  To issue a certain request, the software creates the SETUP stage's DATA packet using the appropriate '''request code''', a valid bmRequestType, the appropriate parameter values (or zero, if not applicable) for wValue and wIndex, and the amount of data bytes to be transfered for wLength.  To the right are the standard USB device request codes, and the remainder of this section discusses each request.

==== SET_ADDRESS ====
The SET_ADDRESS request has the following SETUP DATA packet format:
{| align="center" border="1" cellpadding="5"
|+align="bottom" |'''SET_ADDRESS SETUP DATA Packet Format'''
!bmRequestType
!bRequest
!wValue
!wIndex
!wLength
|- align="center"
|00000000b
|SET_ADDRESS<br />5
|Device Address
|Zero
|Zero
|}

This request does not have a DATA stage, only a SETUP and STATUS stage.

''wValue'' specifies the address to be assigned to the device.  The behavior of a device is undefined when ''wValue'' specifies an address greater than 127.

The exact behavior of the device after the SET_ADDRESS requests depends on the current state of the device:
* When a device is in the default state, a non-zero ''wValue'' causes the device to transition into the address state.  When a device is in the default state, a ''wValue'' of zero has no effect.
* When a device is in the address state, a non-zero ''wValue'' keeps a device in the Address state, but the device responds to the newly set address.  When a device is in the address state, a ''wValue'' of zero transitions the device into the default state.
* When a device is in the configurd state, device behavior is not defined for the SET_ADDRESS request.

This is the only request that is complete after the STATUS stage completes successfully.  After the reset/resume recovery interval (10 milliseconds), a device is expected to be able to complete the STATUS stage of this request within 50 milliseconds.  After the STATUS stage is complete, the device is allowed a 2 millisecond recovery interval before it must be able to accept farther SETUP packets for additional requests.

==== GET_DESCRIPTOR ====
{|border="1" cellpadding="2" style="float:right;clear:right;margin-left:20px;"
|+align="bottom" |'''Standard USB Descriptor Types'''
!Descriptor Type
!Value
|-
|[[#DEVICE|DEVICE]]
|align="center" |1
|-
|[[#CONFIGURATION|CONFIGURATION]]
|align="center" |2
|-
|[[#STRING|STRING]]
|align="center" |3
|-
|[[#INTERFACE|INTERFACE]]
|align="center" |4
|-
|[[#ENDPOINT|ENDPOINT]]
|align="center" |5
|-
|[[#DEVICE_QUALIFIER|DEVICE_QUALIFIER]]
|align="center" |6
|-
|[[#OTHER_SPEED_CONFIGURATION|OTHER_SPEED_CONFIGURATION]]
|align="center" |7
|-
|INTERFACE_POWER
|align="center" |8
|}
The GET_DESCRIPTOR request has the following SETUP DATA packet format:
{| align="center" border="1" cellpadding="5"
|+align="bottom" |'''GET_DESCRIPTOR SETUP DATA Packet Format'''
!bmRequestType
!bRequest
!colspan="2" |wValue
!wIndex
!wLength
|- align="center"
|10000000b
|GET_DESCRIPTOR<br />6
|Descriptor Type
|Descriptor Index
|Zero or<br />Language ID
|Descriptor<br />Length
|}

The high-order byte of ''wValue'' specifies the descriptor type (see table of usb standard descriptor types, to the right).  The low-order byte of ''wValue'' is only used for selecting a specific STRING or CONFIGURATION descriptor, and should be reset to zero otherwise.

The ''wIndex'' field is only used for STRING descriptors to specify the desired language and should be reset to zero for other descriptor types.

Different descriptor types have different lengths which will be discussed soon.  If ''wLength'' is less than the size of the descriptor being returned, then the device only returns the first ''wLength'' bytes of the descriptor data.  If ''wLength'' is larger than the size of the descriptor being returned, than the full descriptor is returned, followed by a short packet (a packet shorter than the maximum data payload size, including a length of 0 bytes).

All USB devices must support requests for DEVICE, CONFIGURATION, and STRING descriptors.  All high-speed devices must support basic operations at full-speed; such devices also support DEVICE_QUALIFIER and OTHER_SPEED_CONFIGURATION descriptors which return the same data that the device would return for DEVICE and CONFIGURATION descriptor requests, respectively, if the device were operating at the speed at which it is not currently operating.

A request for a CONFIGURATION descriptor also returns all the INTERFACE descriptors for the specified configuration descriptor index (i.e, the low-order byte of ''wValue''), as well as all of the ENDPOINT descriptors associated with all of the returned INTERFACE descriptors, all in a single request.

GET_DESCRIPTOR is a valid request for a device in the default, address, or configured state.

==== SET_DESCRIPTOR ====
The SET_DESCRIPTOR request is optional; when it is supported, it may be used to update descriptors or add new ones.

The SET_DESCRIPTOR request has the following SETUP DATA packet format:
{| align="center" border="1" cellpadding="5"
|+align="bottom" |'''SET_DESCRIPTOR SETUP DATA Packet Format'''
!bmRequestType
!bRequest
!colspan="2" |wValue
!wIndex
!wLength
|- align="center"
|00000000b
|SET_DESCRIPTOR<br />7
|Descriptor Type
|Descriptor Index
|Zero or<br />Language ID
|Descriptor<br />Length
|}

The high-order byte of ''wValue'' specifies the descriptor type.  The low-order byte of ''wValue'' is only used for selecting a specific STRING or CONFIGURATION descriptor, and should be reset to zero otherwise.

The ''wIndex'' field is only used for STRING descriptors to specify the desired language and should be reset to zero for other descriptor types.

The ''wLength'' field specifies how many bytes will be transfered from the host to the device.

This request only supports DEVICE, CONFIGURATIOn, and STRING descriptor types.

If this request is not supported, the device responds with a Reuqest Error.

If this request is supported, it is only valid when the device is in the address or configured state; the behavior of the device is undefined if this request is made while the device is in the default state.

==== GET_CONFIGURATION ====
The GET_CONFIGURATION request has the following SETUP DATA packet format:
{| align="center" border="1" cellpadding="5"
|+align="bottom" |'''GET_CONFIGURATION SETUP DATA Packet Format'''
!bmRequestType
!bRequest
!wValue
!wIndex
!wLength
|- align="center"
|10000000b
|GET_CONFIGURATION<br />8
|Zero
|Zero
|One
|}

The device sends a one-byte DATA packet during the DATA phase of the control transfer.  This byte is the value of the current configuration of the device.  A value of zero indicates that the device has not yet been configured (it is in the address state).  Behavior of a device is undefined if this request is issued while the device is in the default state.

==== SET_CONFIGURATION ====
The SET_CONFIGURATION request has the following SETUP DATA packet format:
{| align="center" border="1" cellpadding="5"
|+align="bottom" |'''SET_CONFIGURATION SETUP DATA Packet Format'''
!bmRequestType
!bRequest
!colspan="2" |wValue
!wIndex
!wLength
|- align="center"
|00000000b
|SET_CONFIGURATION<br />9
|''Reserved''
|Configuration Value
|Zero
|Zero
|}

The low-order byte of ''wValue'' specifies the desired configuration value.  The low-order byte of ''wValue'' must either be zero, or it must match the configuration value field of a configuration descriptor returned by the device.  Specifying a configuration value of zero sets the device into the address state.

If the device is in the default state, or if the high-order byte of ''wValue'' is not zero, ''wIndex'' is not zero, or ''wLength'' is not zero, then the behavior after issuing this request is undefined.

If the specified configuration value is neither zero nor a valid configuration value specified by a configuration descriptor of the device, the device responds with a Request Error.

==== GET_INTERFACE ====
The GET_INTERFACE request has the following SETUP DATA packet format:
{| align="center" border="1" cellpadding="5"
|+align="bottom" |'''GET_INTERFACE SETUP DATA Packet Format'''
!bmRequestType
!bRequest
!wValue
!wIndex
!wLength
|- align="center"
|10000001b
|GET_INTERFACE<br />10
|Zero
|Interface
|One
|}

The host uses this request to determine which alternate setting (as described in [[#Functions.2C_Configurations.2C_Interfaces.2C_and_Endpoints|Functions, Configurations, Interfaces, and Endpoints]]) is used for a particular interface of the current configuration.  The device responds with a one-byte long DATA packet during the data phase, the transfered byte being the alternate setting value for the interface specified in this request.

If ''wValue'' is not zero, ''wLength'' is not one, ''wIndex'' specifies an invalid interface, or the device is in the address state, then the device responds with a Request Error.

The behavior of a device in the default state after receiving this request is undefined.

This request is valid for a device in the configured state.

==== SET_INTERFACE ====
The SET_INTERFACE request has the following SETUP DATA packet format:
{| align="center" border="1" cellpadding="5"
|+align="bottom" |'''SET_INTERFACE SETUP DATA Packet Format'''
!bmRequestType
!bRequest
!wValue
!wIndex
!wLength
|- align="center"
|00000001b
|SET_INTERFACE<br />11
|Alternate Setting
|Interface
|Zero
|}

The host uses this request to select an alternate setting (as described in [[#Functions.2C_Configurations.2C_Interfaces.2C_and_Endpoints|Functions, Configurations, Interfaces, and Endpoints]]) to be used for a particular interface of the current configuration.  If the interface specified only supports a default setting, then the device may return a STALL handshake during the STATUS stage of the request.

If the interface or alternate setting do not exist, or if the device is in the address state, then the device responds with a Request Error.  The behavior of the device is undefined if ''wLength'' is not zero, or the device is in the default state.

This is a valid request when the device is in the configured state.

==== CLEAR_FEATURE ====
{| style="clear:right;float:right;margin-left:20px;" border="1" cellpadding="2"
|+align="bottom" |'''Standard USB Feature Selectors'''
!Feature Selector
!Recipient
!Value
|-
|DEVICE_REMOTE_WAKEUP
|align="center" |Device
|align="center" |1
|-
|ENDPOINT_HALT
|align="center" |Endpoint
|align="center" |0
|-
|TEST_MODE
|align="center" |Device
|align="center" |2
|}
The CLEAR_FEATURE request has the following SETUP DATA packet format:
{| align="center" border="1" cellpadding="5"
|+align="bottom" |'''CLEAR_FEATURE SETUP DATA Packet Format'''
!bmRequestType
!bRequest
!wValue
!wIndex
!wLength
|- align="center"
|00000000b<br />00000001b<br />00000010b
|CLEAR_FEATURE<br />1
|Feature Selector
|Zero or<br />Interface or<br />Endpoint
|Zero
|}

The host uses this request to clear or disable a specific feature.

''wValue'' must contain a feature selector (see table of Standard USB Feature Selectors, to the right) which corresponds with the recipient as specified in the ''bmRequestType'' value.

Issuing this request while referencing a feature that cannot be cleared or does not exist, or referencing an interface or endpoint that does not exist will cause the device to respond with a Request Error.

If the device is in the default state, or ''wLength'' is non-zero, then the behavior of the device is undefined.

This request is valid when the device is in the configured state.  When the device is in the address state, this request is only valid when referencing endpoint zero, otherwise the device responds with a Request Error.

The TEST_MODE feature cannot be cleared by this request.

==== SET_FEATURE ====
{| style="clear:right;float:right;margin-left:20px;" border="1" cellpadding="2"
|+align="bottom" |'''Standard USB Test Selectors'''
!Value
!Description
|-
|align="center" |00h
|''Reserved''
|-
|align="center" |01h
|Test_J
|-
|align="center" |02h
|Test_K
|-
|align="center" |03h
|Test_SE0_NAK
|-
|align="center" |04h
|Test_Packet
|-
|align="center" |05h
|Test_Force_Enable
|-
|align="center" |06h-3Fh
|''Reserved for standard test selectors''
|-
|align="center" |3Fh-BFh
|''Reserved''
|-
|align="center" |C0h-FFh
|''Reserved for vendor-specific test modes''
|}
The SET_FEATURE request has the following SETUP DATA packet format:
{| align="center" border="1" cellpadding="5"
|+align="bottom" |'''SET_FEATURE SETUP DATA Packet Format'''
!bmRequestType
!bRequest
!wValue
!colspan="2" |wIndex
!wLength
|- align="center"
|00000000b<br />00000001b<br />00000010b
|SET_FEATURE<br />3
|Feature Selector
|Test selector
|Zero or<br />Interface or<br />Endpoint
|Zero
|}
The host uses this request to set or enable a specific feature.

''wValue'' must contain a feature selector which corresponds with the recipient as specified in the ''bmRequestType'' value.

When ''wValue'' selects the TEST_MODE feature, ''bmRequestType'' and the low-order byte of ''wIndex'' must both be reset to zero.  The high-order byte of ''wIndex'' must be a valid test selector (see table of Standard USB Test Selectors, to the right), or the device respond with Request Error.  The device must set its upstream-facing port into test mode no longer than 3milliseconds after completing the STATUS stage of this request.  In order to exit test mode, the power to the device must be cycled.  A device must support the TEST_MODE feature in the default, address, and configured high-speed device states.

If this request references a feature that does not exist or cannot be set, then the devices responds with a STALL handshake during the STATUS stage.

If an endpoint or interface is specified that does not exist, or if the device is in the address state and an endpoint other than endpoint zero is specified, the device responds with a Request Error.

Besides requests which select the TEST_MODE feature, issuing this request to a device in the default state results in undefined behavior.  A non-zero value for ''wLength'' also results in undefined behavior.

This request is valid when a device is in the configured state, or when the device is in the address state and only endpoint zero is referenced.

==== GET_STATUS ====
The GET_STATUS request has the following SETUP DATA packet format:
{| align="center" border="1" cellpadding="5"
|+align="bottom" |'''GET_STATUS SETUP DATA Packet Format'''
!bmRequestType
!bRequest
!wValue
!wIndex
!wLength
|- align="center"
|10000000b<br />10000001b<br />10000010b
|GET_STATUS<br />0
|Zero
|Zero or<br />Interface or<br />Endpoint
|Two
|}

The host uses this request to learn the status of the recipient, as specified by the ''bmRequestType'' field and, in the case of an interface or endpoint recipient, the ''wIndex'' field.

If ''wValue'' is not zero, ''wLength'' is not two, or ''wIndex'' is non-zero when ''bmRequestType'' specifies a device recipient, or the device is in the default state, then the behavior of the device is undefined.

If this request references an endpoint or interface that does not exist (including any endpoint other than endpoint zero when the device is in the address state), then the device responds with a Request Error.

In response to this request, the device issues a 2 byte long data transfer during the DATA stage to the host.  These two bytes represent the requested status and the meaning depends on the recipient type.

===== Device Recipient =====
When the recipient was a device, the two bytes describe the status as follows:
{| align="center" border="1"
|- align="center"
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">7</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">6</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">5</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">4</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">3</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">2</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">1</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">0</span>'''
|- align="center"
|colspan="6" |''Reserved (reset to zero)''
|Remote<br />Wakeup
|Self<br />Powered
|- align="center"
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">15</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">14</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">13</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">12</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">11</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">10</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">9</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">8</span>'''
|- align="center"
|colspan="8" |''Reserved (reset to zero)''
|}
The ''Self Powered'' field is set to 1 to indicate that the device is currently powered by an external power source, or 0 to indicate that the device is currently running on power supplied by the bus.

The ''Remote Wakeup'' field is set to 0 when the device is reset and indicates whether or not the device is currently enabled to perform remote wakeup signaling (see [[#Remote_Wakeup_Capability|Remote Wakeup Capability]]).  The host may modify the value of the ''Remote Wakeup'' field by issuing either a CLEAR_FEATURE or SET_FEATURE request using the DEVICE_REMOTE_WAKEUP feature selector.

===== Interface Recipient =====
When the recipient was an interface, the two bytes describe the status as follows:
{| align="center" border="1"
|- align="center"
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">7</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">6</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">5</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">4</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">3</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">2</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">1</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">0</span>'''
|- align="center"
|colspan="8" |''Reserved (reset to zero)''
|- align="center"
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">15</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">14</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">13</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">12</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">11</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">10</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">9</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">8</span>'''
|- align="center"
|colspan="8" |''Reserved (reset to zero)''
|}

===== Endpoint Recipient =====
When the recipient was an endpoint, the two bytes describe the status as follows:
{| align="center" border="1"
|- align="center"
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">7</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">6</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">5</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">4</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">3</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">2</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">1</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">0</span>'''
|- align="center"
|colspan="7" |''Reserved (reset to zero)''
|Halt
|- align="center"
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">15</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">14</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">13</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">12</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">11</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">10</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">9</span>'''
|style="padding:15px 15px 15px 15px;" |'''D<span style="vertical-align:sub;">8</span>'''
|- align="center"
|colspan="8" |''Reserved (reset to zero)''
|}

All interrupt and bulk endpoint types must implement the halt feature, otherwise it is optional.  The ''Halt'' field reflects the status of the halt feature of the endpoint.  A value of 0 in the ''Halt'' field indicates that the endpoint is not halted, and a value of 1 in the ''Halt'' field indicates that the endpoint is halted.

The host may set the halt feature of an endpoint with the SET_FEATURE request using the ENDPOINT_HALT feature selector, or the host may clear the halt feature of an endpoint with the CLEAR_FEATURE request using the ENDPOINT_HALT feature selector.  When the CLEAR_FEATURE request is used in this manner, and the endpoint uses a data toggle bit, the data toggle bit is reset to zero.

The default control pipe is not required nor recommended to implement the halt feature, but some devices may choose to use the halt feature on the default control pipe to reflect a functional error condition.  When the halt feature is set on the default control pipe, the device responds with a STALL handshake during the DATA or STATUS stage of all transfers with the exception of the GET_STATUS, CLEAR_FEATURE, and SET_FEATURE standard requests.  Additionally, the device is not required to stall vendor- or class-specific requests when the halt feature is set.

==== SYNCH_FRAME ====
The SYNCH_FRAME request has the following SETUP DATA packet format:
{| align="center" border="1" cellpadding="5"
|+align="bottom" |'''SYNCH_FRAME SETUP DATA Packet Format'''
!bmRequestType
!bRequest
!wValue
!wIndex
!wLength
|- align="center"
|10000010b
|SYNCH_FRAME<br />12
|Zero
|Endpoint
|Two
|}

This request is only used for isochronous endpoints that use implicit pattern synchronization.  That is, some isochronous endpoints require per-frame transfers to vary in size according to a specific pattern (in order to attain an application-specific bit rate, for example).  This call causes the device to send the host a 2 byte value that is the number of the frame where the pattern began.

High-speed isochronous endpoints that support this request must synchronize to the zeroth microframe, as well as have a time notion of classic frames (1 millisecond as opposed to 125 microsecond intervals).

If ''wValue'' is not zero, ''wLength'' is not two, or the device is in the default state, then the behavior of the device is undefined.

If the specified endpoint does not support this request, or the device is in the address state, then the device responds with a Request Error.

This request is valid when the device is in the configured state.

=== Standard USB Descriptors ===
A '''descriptor''' is a data structure with a defined format.  All standard descriptors begin with two bytes.  The first byte specifies the total length of the descriptor in bytes, including the first two mandatory bytes.  The second byte identifies the type of the descriptor.

Some descriptors contain fields which specify an index of a STRING descriptor, but it is optional for a device to support STRING descriptors.  If a device does not support STRING descriptors, then all fields which reference an index of a STRING descriptor should be reset to zero.  Thus, a value of zero in any field that is meant to supply an index of a STRING descriptor indicates that no such STRING descriptor is available.

If the second byte of a descriptor identifies that descriptor as one of the standard USB descriptors, but the first byte of that descriptor specifies a length less than the lengths defined in the USB 2.0 specifications (and, transitively, here), then the descriptor should be rejected by the host.  If the length field reports that the descriptor is longer than expected, then the extra data should be ignored, but still considered part of the descriptor (this is important when the device is returning multiple descriptors, as is the case when the host requests a CONFIGURATION descriptor).

If class- or vendor-specific descriptors use the same format as standard descriptors (i.e, the two mandatory bytes at the beginning of the descriptor), then the class- or vendor-specific descriptors are interleaved within the results when the host requests a CONFIGURATION descriptor.  Otherwise, the class- or vendor-specific descriptors are accessed by passing a class- or vendor-specific descriptor type in a GET_DESCRIPTOR request.

The remainder of this section serves to catalog the standard USB device descriptors and very closely mirrors section 9.6 of the USB 2.0 specifications.  These descriptor definitions supplement the [[#GET_DESCRIPTOR|GET_DESCRIPTOR]] request.

==== DEVICE ====
Every USB device has exactly one DEVICE descriptor.  This descriptor provides general information about the device, as well as information that applies globally to the device and all of its configurations.
{| align="center" border="1"
!Offset
!Field
!Size
!Type
!Description
|- valign="top"
|align="center" |0
|bLength
|align="center" |1
|align="center" |Number
|Size of this descriptor in bytes
|- valign="top"
|align="center" |1
|bDescriptorType
|align="center" |1
|align="center" |Constant
|DEVICE Descriptor Type
|- valign="top"
|align="center" |2
|bcdUSB
|align="center" |2
|align="center" |BCD
|USB Specification Release Number in Binary-Coded Decimal (i.e, 2.10 is expressed as 210h).  Identifies the release of the USB Specification with with the device and its descriptors are compliant.
|- valign="top"
|align="center" |4
|bDeviceClass
|align="center" |1
|align="center" |Class
|Class code (assigned by the USB-IF)
* This field is reset to zero if each interface within a configuration specifies its own class information and the various interfaces operate independently.
* A value of FFh in this field indicates the device class is vendor-specific.
|- valign="top"
|align="center" |5
|bDeviceSubClass
|align="center" |1
|align="center" |SubClass
|Subclass Code (assigned by the USB-IF)
* The subclass code of a device is qualified by the class code of that device.
* If ''bDeviceClass'' is reset to zero, then this field must also be reset to zero.
* When ''bDeviceClass'' is not set to FFh, then all values for this field are reserved for assignment by the USB-IF.
|- valign="top"
|align="center" |6
|bDeviceProtocol
|align="center" |1
|align="center" |Protocol
|Protocol code (assigned by the USB-IF)
* The protocol code of a device is qualified by both the class and subclass codes of that device.
* A value of 00h in this field means that the device may specify class-specific protocols on an interface basis, though this is not a requirement.
* If this field is set to FFh, then the device uses a vendor-specific protocol.
|- valign="top"
|align="center" |7
|bMaxPacketSize0
|align="center" |1
|align="center" |Number
|Maximum packet size for endpoint zero (8, 16, 32, or 64 are the only valid options)
|- valign="top"
|align="center" |8
|idVendor
|align="center" |2
|align="center" |ID
|Vendor ID (assigned by the USB-IF)
|- valign="top"
|align="center" |10
|idProduct
|align="center" |2
|align="center" |ID
|Product ID (assigned by the USB-IF)
|- valign="top"
|align="center" |12
|bcdDevice
|align="center" |2
|align="center" |BCD
|Device release number in binary-coded decimal
|- valign="top"
|align="center" |14
|iManufacturer
|align="center" |1
|align="center" |Index
|Index of STRING descriptor describing manufacturer
|- valign="top"
|align="center" |15
|iProduct
|align="center" |1
|align="center" |Index
|Index of STRING descriptor describing product
|- valign="top"
|align="center" |16
|iSerialNumber
|align="center" |1
|align="center" |Index
|Index of STRING descriptor describing the device's serial number
|- valign="top"
|align="center" |17
|bNumConfigurations
|align="center" |1
|align="center" |Number
|Number of possible configurations
|}

==== DEVICE_QUALIFIER ====
A high-speed capable device that has different device information depending on the speed in which the device operating, then that device must also have a DEVICE_QUALIFIER descriptor.  This descriptor provides information about the device that would change if the device were operating at the alternate speed (i.e, when the device is operating at high-speed, this descriptor provides the differences if the device were operating a full-speed, and vice versa).  This descriptor leaves out fields from the DEVICE descriptor that would not reasonably depend on the speed of the device (e.g, index of the STRING descriptor describing the product).

If a full-speed only device with a ''bcdUSB'' field of at least 0200h in its DEVICE descriptor receives a request for a DEVICE_QUALIFIER descriptor, it must respond with a Request Error.
{| align="center" border="1"
!Offset
!Field
!Size
!Type
!Description
|- valign="top"
|align="center" |0
|bLength
|align="center" |1
|align="center" |Number
|Size of this descriptor in bytes
|- valign="top"
|align="center" |1
|bDescriptorType
|align="center" |1
|align="center" |Constant
|DEVICE_QUALIFIER Descriptor Type
|- valign="top"
|align="center" |2
|bcdUSB
|align="center" |2
|align="center" |BCD
|USB Specification Release Number in Binary-Coded Decimal (i.e, 2.00 is expressed as 200h).  Identifies the release of the USB Specification with with the device and its descriptors are compliant.<br /><br />
This field must be at least 0200h.
|- valign="top"
|align="center" |4
|bDeviceClass
|align="center" |1
|align="center" |Class
|Class code (assigned by the USB-IF)
|- valign="top"
|align="center" |5
|bDeviceSubClass
|align="center" |1
|align="center" |SubClass
|Subclass Code (assigned by the USB-IF)
|- valign="top"
|align="center" |6
|bDeviceProtocol
|align="center" |1
|align="center" |Protocol
|Protocol code (assigned by the USB-IF)
|- valign="top"
|align="center" |7
|bMaxPacketSize0
|align="center" |1
|align="center" |Number
|Maximum packet size for endpoint zero (8, 16, 32, or 64 are the only valid options)
|- valign="top"
|align="center" |8
|bNumConfigurations
|align="center" |1
|align="center" |Number
|Number of possible configurations
|- valign="top"
|align="center" |9
|bReserved
|align="center" |1
|align="center" | -
|Reserved for future uses, must be zero.
|}

==== CONFIGURATION ====
All USB devices have at least one CONFIGURATION descriptor.  The host may request a specific CONFIGURATION descriptor by its descriptor indexx, which is zero based and has ''bNumConfigurations'' (as returned in the DEVICE descriptor) used indices.  That is, the valid values to be used as a descriptor index when requesting a CONFIGURATION descriptor are any integer in the range of 0 to ''bNumConfigurations''-1, inclusive.

Each CONFIGURATION descriptor has at least one INTERFACE descriptor, and each INTERFACE descriptor may have up to 15 ENDPOINT descriptors.  When the host requests a certain CONFIGURATION descriptor, the device returns the CONFIGURATION descriptor followed immediately by the first INTERFACE descriptor, followed immediately by all of the ENDPOINT descriptors for endpoints that the interface defines (which may be none).  This is followed immediately by the next INTERFACE descriptor if one exists, and then by its ENDPOINT descriptors if applicable.  This pattern continues until all the information within the scope of the specific configuration is transfered.

When a device has vendor- or class-specific descriptors that conform to the standard USB descriptor format (that is, the first byte of the descriptor determines the length of the descriptor, and the second byte identifies the type of descriptor), those descriptors are also returned interleaved among the CONFIGURATION, INTERFACE, and ENDPOINT descriptors when the host requests a specific CONFIGURATION descriptor.  Therefore, the system software cannot assume continuous standard descriptors as implied by the previous paragraph; instead, the system software should check the descriptor type and skip that descriptor if it is not a standard descriptor.  The software should also check that standard descriptors report at least the expected length.

Note that the CONFIGURATION descriptor index is not the same as the value ''bConfigurationValue'' in the CONFIGURATION descriptor.  ''bConfigurationValue'' is the value that the host passes as a parameter with the SET_CONFIGURATION request in order to select a particular configuration, whereas this cannot be done using the CONFIGURATION descriptor index.
{| align="center" border="1"
!Offset
!Field
!Size
!Type
!Description
|- valign="top"
|align="center" |0
|bLength
|align="center" |1
|align="center" |Number
|Size of this descriptor in bytes
|- valign="top"
|align="center" |1
|bDescriptorType
|align="center" |1
|align="center" |Constant
|CONFIGURATION Descriptor Type
|- valign="top"
|align="center" |2
|wTotalLength
|align="center" |2
|align="center" |Number
|The total combined length in bytes of all the descriptors returned with the request for this CONFIGURATION descriptor (including CONFIGURATION, INTERFACE, ENDPOINT, class- and vendor-specific descriptors).
|- valign="top"
|align="center" |4
|bNumInterfaces
|align="center" |1
|align="center" |Number
|Number of interfaces supported by this configuration
|- valign="top"
|align="center" |5
|bConfigurationValue
|align="center" |1
|align="center" |Number
|Value which when used as an argument in the SET_CONFIGURATION request, causes the device to assume the configuration described by this descriptor.
|- valign="top"
|align="center" |6
|iConfiguration
|align="center" |1
|align="center" |Index
|Index of STRING descriptor describing this configuration.
|- valign="top"
|align="center" |7
|bmAttributes
|align="center" |1
|align="center" |Bitmap
|Configuration Characteristics
{| border="1" cellpadding="15" align="center"
|D<span style="vertical-align:sub;">7</span>
|D<span style="vertical-align:sub;">6</span>
|D<span style="vertical-align:sub;">5</span>
|D<span style="vertical-align:sub;">4</span>
|D<span style="vertical-align:sub;">3</span>
|D<span style="vertical-align:sub;">2</span>
|D<span style="vertical-align:sub;">1</span>
|D<span style="vertical-align:sub;">0</span>
|}
{| border="0" cellpadding="0" align="center" style="max-width:500px;"
|valign="top" style="min-width:80px;" |'''D<span style="vertical-align:sub;">7</span>'''
|''Reserved, must be set to one for historical reasons''
|-
|valign="top" style="min-width:80px;" |'''D<span style="vertical-align:sub;">6</span>'''
|'''Self-Powered'''
* 0 = Device runs on power supplied by the bus
* 1 = Device provides a local power source, if ''bMaxPower'' is non-zero, the device also may use bus power.
|-
|valign="top" style="min-width:80px;" |'''D<span style="vertical-align:sub;">5</span>'''
|'''Remote Wakeup'''
* 0 = Remote Wakeup not supported
* 1 = Remote Wakeup supported
|-
|valign="top" style="min-width:80px;" |'''D<span style="vertical-align:sub;">4...0</span>'''
|''Reserved, reset to zero''
|}
|- valign="top"
|align="center" |8
|bMaxPower
|align="center" |1
|align="center" |mA
|Maximum power consumption of this device from the bus when fully operational and using this configuration.
* Expressed in units of 2mA (i.e., a value of 50 in this field indicates 100mA).
* A device reports with the ''bmAttributes'' field whether the configuration is bus- or self-powered, but the device status (retrieved with a GET_STATUS request) reports whether the device is currently self-powered.
* If a device is disconnected from an external power source, it may not draw more power from the bus than specified in this field.
* Some devices may be able to operate solely on bus power.  A device that cannot and has lost its external power source will fail the operations it can no longer support.  It is up to the software on the host to determine when this is the case, which may be accomplished with a GET_STATUS request.
|}


==== OTHER_SPEED_CONFIGURATION ====
This descriptor describes the configuration of a high-speed device if it were operating at it's alternative speed.  The host should not request this descriptor unless it already successfully received a DEVICE_QUALIFIER descriptor from the device.  The structure of the OTHER_SPEED_CONFIGURATION is identical to that of the CONFIGURATION descriptor shown above.  The only difference is that the ''bDescriptorType'' field reflects that the descriptor is an OTHER_SPEED_CONFIGURATION descriptor rather than a CONFIGURATION descriptor.

==== INTERFACE ====
INTERFACE descriptors are only returned following a CONFIGURATION descriptor when the host requests a specific CONFIGURATION descriptor; it is not possible to directly request a specific INTERFACE descriptor.  An interface may provide alternate settings within a configuration that allow the endpoints and/or their characteristics to be varied.  A default interface has the ''bAlternateSetting'' field in its INTERFACE descriptor reset to zero.
{| align="center" border="1"
!Offset
!Field
!Size
!Type
!Description
|- valign="top"
|align="center" |0
|bLength
|align="center" |1
|align="center" |Number
|Size of this descriptor in bytes
|- valign="top"
|align="center" |1
|bDescriptorType
|align="center" |1
|align="center" |Constant
|INTERFACE Descriptor Type
|- valign="top"
|align="center" |2
|bInterfaceNumber
|align="center" |1
|align="center" |Number
|Number of this interface.  Zero-based value which identifies the index of this interface in the array of interfaces supported within a configuration.
|- valign="top"
|align="center" |3
|bAlternateSetting
|align="center" |1
|align="center" |Number
|Value used to select the alternate settings described by this INTERFACE descriptor for the interface with the ''bInterfaceNumber'' in the previous field.  This value is zero if this descriptor describes the default settings for a particular interface.
|- valign="top"
|align="center" |4
|bNumEndpoints
|align="center" |1
|align="center" |Number
|Number of endpoints used by this interface, not including endpoint zero.
|- valign="top"
|align="center" |5
|bInterfaceClass
|align="center" |1
|align="center" |Class
|Class code (assigned by the USB-IF)
* A value of zero here is reserved for future standardization.
* If this value is FFh, the interface class is vendor-specific.
* All other values are reserved for assignment by the USB-IF.
|- valign="top"
|align="center" |6
|bInterfaceSubClass
|align="center" |1
|align="center" |SubClass
|Subclass code (assigned by the USB-IF)
* The subclass code in this field is qualified by the value of the ''bInterfaceClass'' field.
* If ''bInterfaceClass'' is reset to zero, then this field must also be reset to zero.
* If ''bInterfaceClass'' is not set to the value of FFh, then all values of this field are reserved for assignment by the USB-IF.
|- valign="top"
|align="center" |7
|bInterfaceProtocol
|align="center" |1
|align="center" |Protocol
|Protocol code (assigned by the USB-IF)
* The protocol code in this field is qualified by the values of the ''bInterfaceClass'' and ''bInterfaceSubClass'' fields.
* If an interface supports class-specific requests, then this field identifies the protocols that the device uses as defined by the specifications of the device class.
* If this field is reset to zero, then the device does not use a class-specific protocol on this interface.
* If this field is set to FFh, then the devices uses a vendor-specific protocol on this interface.
|- valign="top"
|align="center" |8
|iInterface
|align="center" |1
|align="center" |Index
|Index of STRING descriptor describing this interface
|}



==== ENDPOINT ====
Each endpoint used for a particular interface has a descriptor which follows after that particular interface's descriptor when the host requests a specific CONFIGURATION descriptor; the host cannot request a specific ENDPOINT descriptor explicitly.  And ENDPOINT descriptor never describes endpoint zero.
{| align="center" border="1"
!Offset
!Field
!Size
!Type
!Description
|- valign="top"
|align="center" |0
|bLength
|align="center" |1
|align="center" |Number
|Size of this descriptor in bytes
|- valign="top"
|align="center" |1
|bDescriptorType
|align="center" |1
|align="center" |Constant
|ENDPOINT Descriptor Type
|- valign="top"
|align="center" |2
|bEndpointAddress
|align="center" |1
|align="center" |Endpoint
|The address of the endpoint on the USB device described by this descriptor.  This field has the following format:
{| border="1" cellpadding="15" align="center"
|D<span style="vertical-align:sub;">7</span>
|D<span style="vertical-align:sub;">6</span>
|D<span style="vertical-align:sub;">5</span>
|D<span style="vertical-align:sub;">4</span>
|D<span style="vertical-align:sub;">3</span>
|D<span style="vertical-align:sub;">2</span>
|D<span style="vertical-align:sub;">1</span>
|D<span style="vertical-align:sub;">0</span>
|}
{| border="0" cellpadding="0" align="center" style="max-width:500px;"
|valign="top" style="min-width:80px;" |'''D<span style="vertical-align:sub;">7</span>'''
|'''Direction''' (ignored for control endpoints)
* 0 = OUT endpoint
* 1 = IN endpoint
|-
|valign="top" style="min-width:80px;" |'''D<span style="vertical-align:sub;">6...4</span>'''
|''Reserved, reset to zero''
|-
|valign="top" style="min-width:80px;" |'''D<span style="vertical-align:sub;">3...0</span>'''
|'''Endpoint Number'''
|}
|- valign="top"
|align="center" |3
|bmAttributes
|align="center" |1
|align="center" |Bitmap
|This field describes the endpoint's attributes as follows:
{| border="1" cellpadding="15" align="center"
|D<span style="vertical-align:sub;">7</span>
|D<span style="vertical-align:sub;">6</span>
|D<span style="vertical-align:sub;">5</span>
|D<span style="vertical-align:sub;">4</span>
|D<span style="vertical-align:sub;">3</span>
|D<span style="vertical-align:sub;">2</span>
|D<span style="vertical-align:sub;">1</span>
|D<span style="vertical-align:sub;">0</span>
|}
{| border="0" cellpadding="0" align="center" style="max-width:500px;"
|valign="top" style="min-width:80px;" |'''D<span style="vertical-align:sub;">7...6</span>'''
|''Reserved, reset to zero''
|-
|valign="top" style="min-width:80px;" |'''D<span style="vertical-align:sub;">5...4</span>'''
|'''Usage Type''' (Isochronous endpoints only; reserved and reset to zero for other endpoints)
* 00 = Data endpoint
* 01 = Feedback endpoint
* 10 = Implicit feedback data endpoint
* 11 = Reserved
|-
|valign="top" style="min-width:80px;" |'''D<span style="vertical-align:sub;">3...2</span>'''
|'''Synchronization Type''' (Isochronous endpoints only; reserved and reset to zero for other endpoint types)
* 00 = No synchronization
* 01 = Asynchronous
* 10 = Adaptive
* 11 = Synchronous
|-
|valign="top" style="min-width:80px;" |'''D<span style="vertical-align:sub;">1...0</span>'''
|'''Transfer Type'''
* 00 = Control
* 01 = Isochronous
* 10 = Bulk
* 11 = Interrupt
|}
|- valign="top"
|align="center" |4
|wMaxPacketSize
|align="center" |2
|align="center" |Number
|Maximum packet size that this endpoint is capable of sending or receiving.
* For isochronous endpoints, this value is used to reserve bus time; the pipe, however, may not always use all of the reserved bus time.
* Bits 10...0 specify the maximum packet size in bytes.
* For high-speed isochronous and interrupt endpoints, bits 12...11 specify the number of additional transaction opportunities per microframe (see [[#High-Speed.2C_High-Bandwidth_Endpoints|High-Speed, High-Bandwidth Endpoints]]).  The format is as follows:
** 00 = None (1 transaction per microframe)
** 01 = 1 additional (2 transactions per microframe)
** 10 = 2 additional (3 transactions per microframe)
** 11 = Reserved
* Bits 15...13 are reserved and must be reset to zero.
|- valign="top"
|align="center" |6
|bInterval
|align="center" |1
|align="center" |Number
|Interval for polling a device during a data transfer, expressed in units of microframes for high-speed devices, and frames for low- and full-speed devices.  The exact meaning of the value in this field depends on the endpoint type and the operating speed of the device:
* Full- and High-speed isochronous endpoints, and high-speed interrupt endpoints:
** This field must be in the range from 1 to 16.
** This field is used to calculate the period as 2<span style="vertical-align:super;">''bInterval'' - 1</span>.  That is, a value of 4 calculates to 2<span style="vertical-align:super;">4 - 1</span> = 2<span style="vertical-align:super;">3</span> = 8.
* Full- and Low-speed interrupt endpoints:
** This field must be in the range from 1 to 255.
* High-speed bulk and control OUT endpoints:
** This field must be in the range from 0 to 255.
** This field specifies the maximum NAK rate of the endpoint.
** A value of zero indicates that the endpoint never NAKs
** Other values indicate at most 1 NAK each ''bInterval'' number of microframes.
** See [[#PING_Transaction_Protocol|PING Transaction Protocol]]
|}

==== STRING ====
Devices may optionally support STRING descriptors.  If a device does not support STRING descriptors, any field which references the index of a STRING descriptor must be reset to zero.  STRING descriptors use unicode encodings and may support multiple languages.  The host requests a STRING descriptor with the [[#GET_DESCRIPTOR|GET_DECRIPTOR]] request and must pass the 16-bit LANGID (as defined by the USB-IF) of the desired language in the ''wIndex'' field.  The list of currently accepted LANGIDs is located [https://web.archive.org/web/20180829193329/https://www.usb.org/developers/docs/USB_LANGIDs.pdf here].

String index 0 for all languages returns a STRING descriptor that contains an array of all the two-byte LANGID codes that the device supports.

Whether requesting a string or an array of LANGIDs, the data is not NULL-terminated.  Instead, the host determines the length of the data by subtracting 2 from the ''bLength'' field of the descriptor.

When the host requests string index 0, the following descriptor is returned:
{| align="center" border="1"
!Offset
!Field
!Size
!Type
!Description
|- valign="top"
|align="center" |0
|bLength
|align="center" |1
|align="center" |Number
|Size of this descriptor in bytes
|- valign="top"
|align="center" |1
|bDescriptorType
|align="center" |1
|align="center" |Constant
|STRING Descriptor Type
|- valign="top"
|align="center" |2
|wLangID[0]
|align="center" |2
|align="center" |Number
|LANGID code zero
|- valign="top"
|align="center" |...
|align="center" |...
|align="center" |...
|align="center" |...
|align="center" |...
|- valign="top"
|align="center" |N
|wLangID[x]
|align="center" |2
|align="center" |Number
|LANGID code x
|}

When the host requests a valid string index other than string index 0 for a supported LANGID, the following descriptor is returned:
{| align="center" border="1"
!Offset
!Field
!Size
!Type
!Description
|- valign="top"
|align="center" |0
|bLength
|align="center" |1
|align="center" |Number
|Size of this descriptor in bytes
|- valign="top"
|align="center" |1
|bDescriptorType
|align="center" |1
|align="center" |Constant
|STRING Descriptor Type
|- valign="top"
|align="center" |2
|bString
|align="center" |N
|align="center" |Number
|Unicode string
|}

== Typical organization of system software ==
This section discusses how system software is typically, reasonably organized.  This section also serves as an index to the wiki entries which provide, or will provide, farther information and perhaps programming examples.

=== USB Device Drivers ===
As with any device driver, a USB device driver abstracts away from the low-level details on just how a specific device is being accessed, and provides the rest of the system and applications with a common interface (e.g, a file manager shouldn't have to know whether it is dealing with an external versus internal hard drive).

USB device drivers typically implement a certain class of device as per the appropriate specifications.  Such classes of USB devices include, but are not limited to:
* [[USB Mass Storage Class Devices]]
* [[USB Human Input Devices]]

=== USB Driver ===
Even a USB device driver need not be concerned with some of the lower-level details.  For instance, it shouldn't matter to the device driver if a device is connected directly to the root hub, or if it lies behind 3 hubs.  The device driver shouldn't worry about how much power the device needs from the bus.  This is where the USB driver comes in.

The USB driver essentially provides the USB framework interface to device drivers.  The USB driver also handles connect and disconnect events on the USB, as well as determining which device driver is needed (according to the Class, Subclass, and Protocol codes), and if that device driver even exists.

=== USB Hub Driver ===
Although the USB Driver knows some details about the USB topography, the responsibility of hub-specific communication (including split-transactions) is often separated from the USB Driver into another module called the USB Hub Driver.

Depending on the design of the system, the USB Driver might bypass the USB Hub Driver when communicating with devices on the root hub, or the system may use the reserved address of 0 to indicate the root hub to the USB Hub Driver (it appears that Linux does this).

Details on USB Hubs will eventually be discussed in the [[USB Hubs]] wiki entry.

=== Host Controller Driver ===
As a request for a data transfer moves from the device driver, through the USB Driver, and through the USB Hub Driver, the request gains all the information needed for the host controller to generate the appropriate transactions on the bus.  However, depending on the host controller, this information needs to be formatted in a certain way and added for scheduling by the host controller.

This task if given to the host controller driver.  Requests reach the host controller driver in a system-defined format, often called a USB Request Block (URB), or an I/O Request Packet (IRP).

Additionally, host controller drivers are loaded by the PCI subsystem when a corresponding host controller is discovered during PCI enumeration.  The host controller driver is thus also responsible for initializing the host controller and perhaps loading the USB Hub Driver and the USB driver.  Combined, the USB driver, USB hub driver, and the host controller driver make up a USB subsystem.

== See Also ==
=== External Links ===
* [http://www.usb.org/home USB.org]
* [https://usb.org/document-library/usb-20-specification USB Universal Serial Bus Revision 2.0 Specification]
* [https://usb.org/document-library/usb-32-specification-released-september-22-2017-and-ecns Universal Serial Bus Revision 3.2 Specification]
* [http://www.usb.org/developers/wusb/wusb1_1_20100910.zip Wireless USB Specification Revision 1.1]
* [http://www.kernel.org/ The Linux kernel] (things tend to be confusing there, plus you have to be careful with educating yourself from Linux sources if your project isn't GPL'ed).
* [http://www.beyondlogic.org/usbnutshell/usb1.shtml USB in a NutShell] may also interest you. It looks like a really good tutorial giving all the required knowledge to understand any other USB documentation/source code in a couple of HTML pages
* [https://web.archive.org/web/20180829193329/https://www.usb.org/developers/docs/USB_LANGIDs.pdf Currently accepted LANGIDs]
* [http://www.usbmadesimple.co.uk/index.html USB Made Simple]
* [https://www.fysnet.net/the_universal_serial_bus.htm USB: The Universal Serial Bus] is a book on writing device/system drivers for UHCI, OHCI, EHCI, and xHCI with various example devices and available source code.

[[Category:USB]]
[[Category:Buses]]

[[de:Universal_Serial_Bus]]
````

<a id="w001729"></a>

## W001729 — Deadlock

Bron-ID: W001729
Namespace: Main (0)
Revisie: 2749 | Datum: 2007-01-10T09:02:37Z
Bron: https://wiki.osdev.org/Deadlock
Revisie-URL: https://wiki.osdev.org/index.php?title=Deadlock&oldid=2749

````text
==Definition==
{{Quotation|
A set of processes is deadlocked if each process in the set is waiting for an event that only another process in the set can cause.
|Andrew Tanenbaum}}

In simpler terms, deadlock occurs when two or more processes are blocking each other, and none are willing to 'give ground' to allow other processes to progress - thus, all the processes will remain in deadlock indefinitely.

A typical example of this is in resource management, such as when two processes attempt to acquire exclusive access to a pair of files. If Process 1 has exclusive access to File A, and attempts to acquire exclusive access to File B, whilst Process 2 has exclusive access to File B, and wants exclusive access to File A, then both will block waiting for the other to finish and release it's lock.

For a deadlock to occur, certain conditions must hold: 
# A process must be able to acquire exclusive access to a given resource, such that it is the only process allowed access - otherwise, one process cannot impede the progress of another. 
# A process must also be able to acquire such access to one resource, wait for a given period of time (during which other processes may act and acquire locks of their own), then acquire access to another resource without losing the previous exclusive lock - "Hold and Wait". If this is not the case, then a process can have exclusive access and block other processes, or be blocked and waiting for access, but not both simultaneously. 
# A process' exclusive lock most also be immune to preemption. If an external agent is capable of removing the process' exclusive access to a resource, it becomes possible for another process to acquire that resource, and the deadlock ends. 
# A process will not stop from trying to acquire a lock once it has started doing so. If it can, it is able to continue doing other operations and consequently is no constituting to an deadlock (as it is not longer waiting)
# It must also be the case, as stated above, that there exists a circle of processes that require access to a resource locked by the next process in the circle. Obviously, deadlock cannot occur where only one process exists, or where the resources accessible by different threads is otherwise controlled. For example, Process 1 can lock resources A and B, Process 2 can lock B and C, and Process 3 can lock C and D - no other access is allowed. In this case, it is not possible for a circle of blocking processes to form, and deadlock cannot occur.

==Prevention==
Preventing deadlock is possible by preventing at least one of the conditions above. The first is straightforward - if resources cannot be locked, deadlock is impossible, though this 'solution' is often not viable. Memory space, for example, must be exclusively assigned, to prevent processes corrupting each other's data. A more intelligent locking system can be employed in some cases, however - files may have an exclusive write lock, and still be readable, preventing many potential deadlocks. Wait-Free synchronization studies this kind of approach.

The second condition can be solved by requiring that a processes relinquish all locked resources before accessing more, allowing other processes an opportunity to acquire those resources and to proceed. This can either be implemented as limiting the number of simultaneous locks held by a resource to one (so it must release the last lock it held before acquiring another), or by requiring the process to declare all it's lock atomically - that is, rather than lock A, wait, then lock B, a process must lock A and B in a single call (or lock A, wait, release A and then lock A and B). Thus, if another process holds either resource, the requesting process gets no locks, and must wait until all resources become free, and it can proceed without impedance. However, this requires either knowledge of all required resources in advance, which is not always possible, or that the process keep requesting and releasing resources.

The viability of preventing the third condition depends on the resources in question, and whether it is possible to preempt a process' access to the resource, and later restore it to the state it was in before the preemption. For example, if two processes each occupy half of the available memory space and both request more, deadlock can occur. However, one process may be preempted by paging it into virtual memory, allowing the other process to continue, before paging the preempted process back into memory - in this case, preemption is transparent and the process never knows that it lost exclusive access. Alternately, preemption may simply inform the process that it has lost access and require the process to deal with the fallout. In some cases, though, preemption is not possible or safe - preempting a print job half way through is simply a waste of ink. Also, preemption requires a trigger - there must therefore be a means to detect deadlocks, even if this is as simple as causing locks held by a blocked process to timeout.

The fourth condition is similar to the third, in that processes can be notified of potential deadlocks. However, no locks are released which can make the situation easier to handle for the process in question.

The final condition requires careful planning of the resources that each process can lock. If each resource is only accessible by a single process, which mediates access by other threads (e.g. a print spooler) then deadlocks are prevented. However, this requires much more pre-planning, and is often more trouble than the risk of deadlocks - if 10 processes exist, and each may only use one tenth of the memory space, or may only run on one tenth of the processor time, then resource efficiency is seriously compromised to prevent deadlocks that will almost certainly never occur. Resource ordering is also a powerful means of preventing deadlocks. If locks can be numbered, and the program will only acquire them in increasing order, there can not be any deadlocks since a program will not be able to start waiting on any program that has only lower-numbered locks, and hence no waiting cycles can exist.

[[Category:Synchronization]]
````

<a id="w001730"></a>

## W001730 — Category:Task Models

Bron-ID: W001730
Namespace: Category (14)
Revisie: 2603 | Datum: 2006-12-29T23:24:59Z
Bron: https://wiki.osdev.org/Category:Task_Models
Revisie-URL: https://wiki.osdev.org/index.php?title=Category%3ATask_Models&oldid=2603

````text
[[Category:Processes and Threads]]
````

<a id="w001731"></a>

## W001731 — Virtual 8086 Mode

Bron-ID: W001731
Namespace: Main (0)
Revisie: 30836 | Datum: 2026-07-02T03:27:33Z
Bron: https://wiki.osdev.org/Virtual_8086_Mode
Revisie-URL: https://wiki.osdev.org/index.php?title=Virtual_8086_Mode&oldid=30836

````text
Virtual 8086 mode is a sub-mode of [[Protected mode]]. In short, Virtual 8086 mode is whereby the CPU (in protected mode) is running a "Emulated" 16bit Real Mode machine.

==Entering V86==

The CPU is executing in virtual 86 mode when the VM bit (bit 17) is set in the ''EFLAGS'' register. If you want to enter virtual 86 mode you must set this bit to 1.
A way of modifying the EFLAGS register is to use the ''PUSHF'' and ''POPF'' instructions, but these instructions are for user space and cannot modify supervisor flags. The only way to set the VM flag is to use the iret instruction. This instruction is normally used to return from an interrupt. When ''IRET'' is called with the VM=1 in the stack ''EFLAGS'' (the context being returned to is V86), the interrupt stack frame will contain segments ''ES'', ''DS'', ''FS'', and ''GS'', so that they can be set before entry.

A task gate can also be used to enter V86. This allows setting the segment registers. It is probably not necessary to do it this way unless the OS is using hardware multitasking.

[[File:Fig15-3.gif]]

==V86 Problem==

The most common problem with v86 mode is that you can't enter Protected mode from inside of a v86 task. In other words, if you are running Windows or have emm386 in memory, you can't do a "raw" switch into protected mode (it causes an exception). DOS extenders worked around that problem using either VCPI or DPMI interfaces to switch into pmode (actually, promoting their V86 task as a 'regular' user task). For an OS programmer such interfaces are simply useless as they're part of another OS.

There are a few other more "technical" problems people have when using v86 mode, mostly because v86 has some instructions "emulated" by what's known as a v86-monitor program, as the CPU is in protected mode, some instructions are high up on the security/protection level and running those directly would cause no-end of trouble for the OS.

==Detecting v8086==

<code>EFLAGS.VM</code> is NEVER pushed onto the stack if the V86 task uses <code>PUSHFD</code>. You should check if <code>CR0.PE = 1</code> and then assume it's V86 if that bit is set.

<syntaxhighlight lang="nasm">
detect_v86:
   smsw    ax
   and     eax, 1           ; CR0.PE bit
   ret
</syntaxhighlight>

VM mode detection is mainly useful when writing DOS extenders or other programs that could be started either in plain real mode or in virtual mode from a protected mode system. An ''ordinary'' [[bootloader]] should not worry about this since the [[BIOS]] will not set up VM86 to read the bootsector.

==Usage==
VM86 can be very useful if one needs access to BIOS functions while in Protected mode. It is in fact necessary in order to set up video mode, as many modern card/chipsets lack support for the VBE3 protected mode interface, so setting up a VM86 task that will perform the 'set video mode' call is the preferred method.

==Kernels below 1MB==
It has been suggested that you map your kernel to a 'high' logical address (e.g. 0xC0000000) to avoid VM86 tasks interfering with it. This is especially important with large kernels which leave no room for VM86 code below 1MB, or when larger programs are expected to run within the VM86 box.

If all that is needed is a BIOS interrupt wrapper, then the following should work:
# ensure that your 16bits code is on a separate page from any 32 bits code
# enable paging
# make kernel pages unwritable (and unreadable ?) for DPL3 and allow user-access only to those pages that contains your 16 bits code and pages that contains BIOS code or data.

==Using VM86 for disk access==
Though theoretically possible, it is probably not a good idea. Most BIOS disk access will include IRQ handlers, DMA transfers (uncontrollable from within your VM monitor), and may stick in VM86 task while the BIOS waits for an interrupt response while a 'good' driver would have let the CPU free for other processes.

Windows 9x suffered from system freezing during disk access. Often due to an INT13-through-VM86 problem.

==See Also==
===Articles===
*[[Virtual Monitor]]
===Threads===
*[[Topic:8437|Creating vm86 task]]
*[[Topic:8477|VM86 and INT10h]]
*[[Topic:8949|kernel location & VM86]]
===External Links===
*{{IntelManualLink|15}}
*http://osdev.berlios.de/v86.html - by Tim Robinson ([https://web.archive.org/web/20140111115023/http://osdev.berlios.de/v86.html Wayback Machine Link])
*[https://web.archive.org/web/20061212050544/http://my.execpc.com/~geezer/osd/pmode/v86mm.zip A virtual-8086 mode monitor] - by Chris Giese
*[http://126.sytes.net/tmp/x86emu.zip x86emu] - a BSD style licensed vitual-8086 mode ''emulator'' - very different from a ''monitor''.
*[http://gitorious.org/x86emu/] - x86emu and several other projects. See mdt for code getting VBE modes.
*[http://www.rohitab.com/discuss/topic/35103-switch-between-real-mode-and-protected-mode/ Protected Mode BIOS Call Functionailty v2.0] - by Napalm
[[Category:X86 CPU]]
[[Category:Operating Modes]]
[[de:Virtual_8086_Mode]]
````

<a id="w001732"></a>

## W001732 — Virtual-8086 Monitor

Bron-ID: W001732
Namespace: Main (0)
Revisie: 30962 | Datum: 2026-09-22T13:17:40Z
Bron: https://wiki.osdev.org/Virtual-8086_Monitor
Revisie-URL: https://wiki.osdev.org/index.php?title=Virtual-8086_Monitor&oldid=30962

````text
{{Historical}}

The Virtual-8086 Monitor is a piece of code you need to set up and control tasks in [[Virtual 8086 Mode]].
==Requirements==
* [[Protected mode]] must be enabled
* [[Interrupt Service Routines]] must be supported, and more specifically a working exception handler for GPF.

==Workings==
Well, you mainly have two flavors: either you can create a standalone task that will run as any other task in your OS but usually operates in V86 mode, or you can temporarily switch to V86 mode from the current task, do a simple command and come back.

What you have to know is that some operations are prohibited in V86 mode, but that legacy code like BIOS or DOS programs will issue them nonetheless. So you'll need to hook the GPF handler so that it detects the faulty operation comes from a virtual mode task and defers execution to the monitor.
How do i set up a virtual task ?

There are a few things that distinguish the set up of a VM task from the setup of any "ordinary" process in your OS:

# If you are using paging, you need to make sure that the [[BIOS]], the real-mode [[IVT]] and any BIOS-related data (e.g. video ram at 0xb8000) is mapped at expected virtual address. This can be done simply by [[Identity Paging]] the first megabyte of this process' address space or you may wish to give private copies of those locations to different VM tasks.
# In your Task Control Block, when you set up the registers on the stack image for the initial run, you have to initialize some additional flags (check out Tim Robinson's tutorial, really). Make sure you at least have EFLAGS=VM|IF, that is, 0x20202. The ip/cs value has to be located inside an area which can be accessed by real mode and you should better have the stack aligned to some 0xffff address.
# You will also need to set up some additional fields: the segment registers, which the processor pops off the stack upon return to a vm86 process Best you make a stack image structure for vm86 tasks too: say vm86_context_t.
# If you want to allow V86 code to access ports without a GPF, you will need to extend the TSS by 8192 bytes (enough for 65,536 ports with one bit per port), point the I/O map field to the start of the bitmap, and set all the bits to zeroes.

Mark II: for the region from 0x100000 to 0x1fffef (IIRC) you can map any pages you want. Only the first MB needs to be identity mapped.

;Task Control Block
: the structure your kernel uses for Task bookkeeping: the processor registers dump, the page directory, time to run, priority, etc.
==Role of General Protection Faults==
Basically, every time the CPU requires the intervention of the Virtual-8086 Monitor, it will raise a GPF exception. Once you detected the exception is due to a virtual task, you call the monitor's GPF handler.

There, you need to read what's the currently tried instruction (beware prefixes like ES:, REPNE, etc.) and decide how you can emulate it...

Opcodes to be supported include:

* 0x9C (pushf) and 0x9D (popf)
* 0xCD (int nn) and 0xCF (iret)
* INx/OUTx (0xE4-0xE7, 0x6C-0x6F, 0xEC-0xEF) unless you set IO permission bitmap/IOPL accordingly
* 0xFA (cli) and 0xFB (sti)

==How should I proceed?==

Your monitor will have to perform operations such as faking interrupts, checking instructions and things alike. A good practice would be to write a couple of 'core' functions that will perform simple operations like pushing a value on the virtual task's stack, reading a word at a given segment:offset (like in [[Real Mode]]), fetching the currently executed byte from virtual CS:IP, etc.

The list of such methods in Chris Giese's monitor, for instance, include

* <tt>unsigned peekb(unsigned seg, unsigned off);</tt> which will return a byte located at seg:off
* <tt>unsigned peekw(unsigned seg, unsigned off);</tt> same for a word
* <tt>void pokeb(unsigned seg, unsigned off, unsigned val);</tt> which will write a byte to seg:off
* <tt>void pokew(unsigned seg, unsigned off, unsigned val);</tt> same with a word
* <tt>void v86_push16(uregs_t *regs, unsigned value);</tt> which will tweak the registers image to push a (16 bits) value on stack.
* <tt>void v86_int(uregs_t *regs, unsigned int_num);</tt> which actually call an INT in vmode.

==Hardware interrupts without IF-related instructions==

There are two Intel provided tricks you may use:

* Set IOPL=3 in your initialization. That will not affect IN/OUT instructions, but instead allows the VM task to toy with IF flags itself. Take care that IRQs and software interrupts will then go directly to the IDT (e.g. the IVT is ignored), so you may need to write code in your pm IRQ handler that will edit the CS:IP values according to that IVT.
* Use "Virtual Mode Extensions", which will allow you to give the TSS a "interrupt redirection bitmap", telling which interrupt should be processed in virtual mode using the IVT and which should be processed in protected mode using the IDT. VME aren't available on QEMU, though.

==See Also==
===External Links===
* http://alexfru.chat.ru/epm.html#v86monitor
* http://osdev.berlios.de/v86.html - TimRobinson's vm86-tutorial (dead link, available on archive.org at https://web.archive.org/web/20090719085533/http://osdev.berlios.de/v86.html)
* http://my.execpc.com/~geezer/osd/pmode/v86mm.zip - Chris Giese's YAV86MM
* http://oslib.cvs.sourceforge.net/viewvc/oslib/oslib/xlib/vm86.c?view=markup - OSLib's basic vm86 code (pages-tuning required)

===Forum===
* [[Topic:9943|Problem with v86 mode and INT n instruction]]
* [[Topic:10287|Issues while implementing VM86 support]]

[[Category:X86 CPU]]
[[Category:Operating Modes]]
````

<a id="w001733"></a>

## W001733 — Template:OSFaq

Bron-ID: W001733
Namespace: Template (10)
Revisie: 2629 | Datum: 2006-12-31T07:34:17Z
Bron: https://wiki.osdev.org/Template:OSFaq
Revisie-URL: https://wiki.osdev.org/index.php?title=Template%3AOSFaq&oldid=2629

````text
<includeonly>
<div style="border: 1px solid #ccc; background-color: #eee; color: #000; text-align: center">
This article needs to include information from the old OsFaq2-wiki: [[mtfaq:{{{1}}}|{{{1}}}]].
</div>

[[Category:Articles needing information from OSFaq]]
</includeonly>
<noinclude>
All articles Including this template get added to [[:Category:Articles needing information from OSFaq]]
==Usage==
<pre>{{OSFaq|ArticleName}}</pre>
==Preview==
<div style="border: 1px solid #ccc; background-color: #eee; color: #000; text-align: center">
This article needs to include information from the old OsFaq2-wiki: ArticleName.
</div>
</noinclude>
````

<a id="w001736"></a>

## W001736 — Talk:Protected Mode

Bron-ID: W001736
Namespace: Talk (1)
Revisie: 5293 | Datum: 2008-03-26T11:37:12Z
Bron: https://wiki.osdev.org/Talk:Protected_Mode
Revisie-URL: https://wiki.osdev.org/index.php?title=Talk%3AProtected_Mode&oldid=5293

````text
I was reading up on Bootloaders when a red link jumped out at me.  I thought I could help by filling it in.  As it turns out, this page already exists.  The Bootloaders page links to [[Protected mode]] not [[Protected Mode]] (note the CamelCase).  The forum users seem to object to CamelCase, so I thought someone ought to know about this.  I would have changed it myself, except that there are other pages that link to this one, and I didn't want to be yelled at.  Also, I don't know how to add redirects.

While I was attempting to add a Protected mode page, I accidently added it under [[OSDevWiki:Help]] because when I clicked the help link it hadn't been created yet.  Once again - as I know nothink - It might not be a bad idea to get that page up.

--[[User:Mr.Confuzed|Mr.Confuzed]] 18:15, 2 January 2007 (CST)
:I deleted [[OSDevWiki:Help]] for you (you can do this with the delete button near the top of every page). I also moved [[Protected Mode]] to [[Protected mode]]. Moving the page can be done by navigating to the existing page you wish to move, and clicking the move button (just right of delete). When moving a page MediaWiki automatically places a redirect from the old location to the new one. -[[User:Jhawthorn|Jhawthorn]] 23:00, 2 January 2007 (CST)
````
